//! `app.hubspot.upsert_contact` — Create or update contact (HubSpot).

use il_sdk::prelude::*;
use reqwest::Method;

use crate::rest;

const API: &str = "https://api.hubapi.com";

/// Create or update contact (HubSpot) node.
#[derive(Debug, Default)]
pub struct AppHubspotUpsertContact;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.hubspot.upsert_contact",
        Category::App,
        "Create or update contact (HubSpot)",
        "Creates a HubSpot contact, or updates the one with the same email.",
    )
    .icon("plug")
    .status(NodeStatus::Ready)
    .app("hubspot")
    .external()
    .credential(cred("hubspot", "hubspot", CredentialKind::ApiKey, "HubSpot private app token", &[]))
    .settings_schema(json!({
        "type": "object",
        "required": ["email"],
        "properties": {
            "email": { "type": "string", "title": "Email", "x-widget": "expression" },
            "properties": {
                "type": "object",
                "title": "Other properties",
                "description": "HubSpot property name → value, e.g. firstname, lastname, company, phone."
            }
        }
    }))
    .example(
        "Save a lead",
        json!({ "email": "{{ item.email }}", "properties": { "firstname": "{{ item.first_name }}", "company": "{{ item.company }}" } }),
    )
});

#[async_trait]
impl NodeExecutor for AppHubspotUpsertContact {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let email = rest::setting(ctx, "email")?.trim().to_lowercase();
        if !email.contains('@') {
            return Err(NodeError::Failed(format!("{email} is not an email address")));
        }
        let mut properties = ctx.settings["properties"].as_object().cloned().unwrap_or_default();
        for value in properties.values_mut() {
            // HubSpot takes every property value as a string.
            if !value.is_string() {
                *value = Value::String(value.to_string());
            }
        }
        properties.insert("email".into(), json!(email));
        let body = json!({ "inputs": [{ "idProperty": "email", "id": email, "properties": properties }] });
        let token = rest::api_key(ctx, "hubspot", "HubSpot").await?;
        let url = format!("{}/crm/v3/objects/contacts/batch/upsert", rest::base(ctx, API));
        let contact = rest::once(ctx, "hubspot:contact", async {
            let result = rest::call(ctx, "HubSpot", Method::POST, &url, &token, &[], Some(&body)).await?;
            let contact = &result["results"][0];
            Ok(json!({ "id": contact["id"], "created": contact["new"], "email": email }))
        })
        .await?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["hubspot"] = contact;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppHubspotUpsertContact)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::AppHubspotUpsertContact;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&AppHubspotUpsertContact);
    }

    #[tokio::test]
    async fn c05_upserts_by_email() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/crm/v3/objects/contacts/batch/upsert"))
            .and(header("authorization", "Bearer pat-na1"))
            .and(body_partial_json(json!({"inputs": [{
                "idProperty": "email",
                "id": "ada@example.com",
                "properties": {"email": "ada@example.com", "firstname": "Ada", "score": "9"}
            }]})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": [{"id": "501", "new": true}]})))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(AppHubspotUpsertContact)
            .settings(json!({
                "email": " Ada@Example.com ", "properties": {"firstname": "Ada", "score": 9}, "_base": server.uri()
            }))
            .credential("hubspot", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "hubspot".into(),
            secret: json!({"api_key": "pat-na1"}),
            meta: json!({}),
        });
        let out = harness.run().await.unwrap();
        assert_eq!(
            out.ports["out"][0].json["hubspot"],
            json!({"id": "501", "created": true, "email": "ada@example.com"})
        );
    }

    #[tokio::test]
    async fn c05_rejects_a_value_that_is_not_an_email() {
        let harness = NodeHarness::new(AppHubspotUpsertContact).settings(json!({"email": "Ada"}));
        assert!(harness.run().await.unwrap_err().to_string().contains("not an email"));
    }
}
