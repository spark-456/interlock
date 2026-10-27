//! Agent tool for a selected OpenAPI operation.

use std::collections::BTreeSet;
use std::sync::Arc;

use async_trait::async_trait;
use il_core::Flow;
use il_sdk::{
    SafeHttp,
    services::{AgentTool, Credential, Services, ToolError},
};
use serde_json::{Map, Value, json};

use super::{
    request, source,
    spec::{self, Operation},
};

/// Builds the selected OpenAPI tools for a flow using its own credential bindings.
pub async fn for_flow(flow: &Flow, services: &Services) -> Result<Vec<Arc<dyn AgentTool>>, ToolError> {
    let mut tools: Vec<Arc<dyn AgentTool>> = Vec::new();
    let mut names = BTreeSet::new();
    let mut selections = Vec::new();
    for node in flow.nodes.iter().filter(|node| !node.disabled) {
        if node.node_type == "data.openapi" {
            let reference =
                node.settings["spec"].as_str().ok_or_else(|| ToolError::InvalidArguments("missing spec".into()))?;
            let selected = node.settings["operationId"]
                .as_str()
                .ok_or_else(|| ToolError::InvalidArguments("missing operationId".into()))?;
            selections.push((reference, selected, node.credentials.get("api"), node.settings["allowRisky"] == true));
        } else if node.node_type == "ai.agent" && node.settings.get("apiTools").is_some() {
            let api = &node.settings["apiTools"];
            let reference = api["spec"].as_str().ok_or_else(|| ToolError::InvalidArguments("missing spec".into()))?;
            let operations =
                api["operations"].as_array().ok_or_else(|| ToolError::InvalidArguments("missing operations".into()))?;
            for selected in operations {
                let selected =
                    selected.as_str().ok_or_else(|| ToolError::InvalidArguments("invalid operation".into()))?;
                selections.push((reference, selected, node.credentials.get("api"), false));
            }
        }
    }
    for (reference, selected, credential_id, allow_risky) in selections {
        let document =
            source::load(reference, &services.http, services.files.as_ref()).await.map_err(ToolError::Failed)?;
        let operation = spec::parse(&document)
            .map_err(ToolError::InvalidArguments)?
            .into_iter()
            .find(|operation| operation.id == selected)
            .ok_or_else(|| ToolError::InvalidArguments(format!("operation not found: {selected}")))?;
        if operation.risky && !allow_risky {
            continue;
        }
        if !names.insert(operation.id.clone()) {
            continue;
        }
        let credential = match credential_id {
            Some(id) => Some(
                services
                    .store
                    .credential(&flow.workspace_id, id)
                    .await
                    .map_err(|e| ToolError::Failed(e.to_string()))?
                    .ok_or_else(|| ToolError::Failed("bound credential not found".into()))?,
            ),
            None => None,
        };
        tools.push(Arc::new(ApiTool::new(operation, services.http.clone(), credential, allow_risky)?));
    }
    Ok(tools)
}

/// A selected operation exposed to an Agent.
pub struct ApiTool {
    operation: Operation,
    http: SafeHttp,
    credential: Option<Credential>,
}

impl ApiTool {
    /// Creates a tool only when its risk policy allows the operation.
    pub fn new(
        operation: Operation,
        http: SafeHttp,
        credential: Option<Credential>,
        allow_risky: bool,
    ) -> Result<Self, ToolError> {
        if operation.risky && !allow_risky {
            return Err(ToolError::InvalidArguments("destructive operation requires allowRisky".into()));
        }
        Ok(Self { operation, http, credential })
    }
}

#[async_trait]
impl AgentTool for ApiTool {
    fn name(&self) -> &str {
        &self.operation.id
    }
    fn description(&self) -> &str {
        &self.operation.description
    }
    fn schema(&self) -> Value {
        let mut properties = Map::new();
        let mut required = Vec::new();
        for parameter in &self.operation.parameters {
            properties.insert(parameter.name.clone(), parameter.schema.clone());
            if parameter.required {
                required.push(parameter.name.clone());
            }
        }
        if let Some(schema) = &self.operation.body_schema {
            properties.insert("body".into(), schema.clone());
        }
        json!({"type":"object","properties":properties,"required":required})
    }
    async fn invoke(&self, args: Value) -> Result<Value, ToolError> {
        request::invoke(&self.http, &self.operation, &args, self.credential.as_ref()).await.map_err(ToolError::Failed)
    }
}

#[cfg(test)]
mod tests {
    use il_core::Flow;
    use il_sdk::services::FileStore;
    use il_sdk::services::{AgentTool, Credential};
    use il_sdk::testkit::FakeServices;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    use super::*;

    #[tokio::test]
    async fn k02_flow_tools_expose_selected_safe_operations() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/invoices/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"paid":true})))
            .mount(&server)
            .await;
        let fakes = FakeServices::default();
        let spec = json!({"openapi":"3.1.0","servers":[{"url":server.uri()}],"paths":{
            "/invoices/{invoiceId}":{
                "get":{"operationId":"getInvoice","parameters":[
                    {"name":"invoiceId","in":"path","required":true,"schema":{"type":"string"}}]},
                "delete":{"operationId":"deleteInvoice"}
            }
        }});
        let file = fakes.files.put("api.json", "application/json", spec.to_string().into()).await.unwrap();
        let flow: Flow = serde_json::from_value(json!({"id":"flow_1","workspaceId":"ws_a","version":1,
        "name":"Billing","kind":"app","edges":[],"nodes":[
            {"id":"get","type":"data.openapi","name":"Get invoice",
                "settings":{"spec":file.id,"operationId":"getInvoice"}},
            {"id":"delete","type":"data.openapi","name":"Delete invoice",
                "settings":{"spec":file.id,"operationId":"deleteInvoice"}}
        ]}))
        .unwrap();
        let tools = for_flow(&flow, &fakes.services()).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name(), "getInvoice");
        assert_eq!(tools[0].invoke(json!({"invoiceId":"42"})).await.unwrap()["paid"], true);
    }

    #[tokio::test]
    async fn k02_agent_tool_uses_credential_and_enforces_risk() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/invoices/42"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"paid":true})))
            .mount(&server)
            .await;
        let operation = Operation {
            id: "getInvoice".into(),
            description: "Get invoice".into(),
            method: "GET".into(),
            base_url: server.uri(),
            path: "/invoices/{invoiceId}".into(),
            parameters: vec![super::super::spec::Parameter {
                name: "invoiceId".into(),
                location: "path".into(),
                required: true,
                schema: json!({"type":"string"}),
            }],
            body_schema: None,
            example_body: None,
            fixed_query: None,
            risky: false,
        };
        let credential = Credential {
            id: "cred_test".into(),
            app: "openapi".into(),
            secret: json!({"api_key":"test-key"}),
            meta: json!({}),
        };
        let tool = ApiTool::new(operation.clone(), SafeHttp::permissive_for_tests(), Some(credential), false).unwrap();
        assert_eq!(tool.schema()["required"], json!(["invoiceId"]));
        assert_eq!(tool.invoke(json!({"invoiceId":"42"})).await.unwrap()["paid"], true);
        let mut destructive = operation;
        destructive.risky = true;
        assert!(ApiTool::new(destructive, SafeHttp::permissive_for_tests(), None, false).is_err());
    }

    #[tokio::test]
    async fn r05_agent_declares_selected_openapi_tools_without_execution_edges() {
        let fakes = FakeServices::default();
        let spec = include_str!("../../../../fixtures/apis/larkspur.openapi.json");
        let file = fakes.files.put("larkspur.openapi.json", "application/json", spec.into()).await.unwrap();
        let flow: Flow = serde_json::from_value(json!({
            "id":"flow_1","workspaceId":"ws_a","version":1,"name":"Support","kind":"app","edges":[],
            "nodes":[{"id":"agent","type":"ai.agent","name":"Billing",
                "settings":{"instructions":"Check invoices","apiTools":{"spec":file.id,
                    "operations":["lookup_customer","list_invoices","delete_customer"]}}}]
        }))
        .unwrap();
        let tools = for_flow(&flow, &fakes.services()).await.unwrap();
        let names: Vec<_> = tools.iter().map(|tool| tool.name()).collect();
        assert_eq!(names, ["lookup_customer", "list_invoices"]);
    }
}
