//! The AI app kit (AR-9): checks that an app has safety rules, writes a test suite of knowledge
//! questions and tool tasks from its goal, runs each case through the flow, and judges the answers.

use il_core::{Flow, Item, Json};
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelError, ModelGateway, ModelRole};
use il_sdk::services::runner::{FlowRunner, StartRun};
use serde::Deserialize;
use serde_json::json;

use crate::catalog::Catalog;
use crate::testfix::Failure;

const GUARDRAILS: &str = "ai.guardrails";
/// Cases in a generated suite.
const MAX_CASES: usize = 5;

/// One test case: a user message and what a good answer does.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Case {
    /// `knowledge` (answered from documents) or `tool` (needs an API call).
    pub kind: String,
    pub input: String,
    pub expect: String,
}

#[derive(Deserialize)]
struct Suite {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Verdict {
    pass: bool,
    #[serde(default)]
    reason: String,
}

/// A failure when the catalog offers safety rules and the app has none.
pub fn missing_guardrails(flow: &Flow, catalog: &Catalog) -> Option<Failure> {
    let offered = catalog.get(GUARDRAILS).is_some();
    (offered && !flow.nodes.iter().any(|n| n.node_type == GUARDRAILS)).then(|| Failure {
        node_id: None,
        problem: format!("the app has no safety rules; add an `{GUARDRAILS}` step before the answer"),
    })
}

/// Writes up to [`MAX_CASES`] cases for the app from `brief`; returns them with their cost.
pub async fn suite(models: &dyn ModelGateway, flow: &Flow, brief: &str) -> Result<(Vec<Case>, f64), ModelError> {
    let text = json!({"type": "string"});
    let schema = json!({"type": "object", "required": ["cases"], "properties": {"cases": {"type": "array",
        "items": {"type": "object", "required": ["kind", "input", "expect"], "properties": {
            "kind": {"type": "string", "enum": ["knowledge", "tool"]}, "input": text, "expect": text}}}}});
    let prompt = format!(
        "{brief}\n\nThe AI app \"{}\" was built for this. Write {MAX_CASES} test cases a real user would send: \
         questions its knowledge should answer and tasks that need its tools. For each, say in one sentence what a \
         good answer does. Reply with JSON: {{\"cases\": [{{\"kind\": \"knowledge\" | \"tool\", \"input\": \"…\", \
         \"expect\": \"…\"}}]}}",
        flow.name
    );
    let req = ChatRequest {
        messages: vec![ChatMessage::user(prompt)],
        json_schema: Some(schema),
        temperature: Some(0.3),
        ..Default::default()
    };
    let resp = models.chat(ModelRole::Architect, req).await?;
    let cases = parse::<Suite>(&resp.content).map(|s| s.cases).unwrap_or_default();
    Ok((cases.into_iter().take(MAX_CASES).collect(), resp.usage.cost_usd))
}

/// Sends each case to the app and judges the answer with the Fast model. Returns how many
/// passed, a failure per failed case, and the judging cost.
pub async fn run(
    models: &dyn ModelGateway,
    runner: &dyn FlowRunner,
    flow_id: &str,
    cases: &[Case],
) -> (u32, Vec<Failure>, f64) {
    let (mut passed, mut failures, mut cost) = (0, Vec::new(), 0.0);
    for case in cases {
        let req = StartRun {
            flow_id: flow_id.into(),
            version: None,
            trigger: "test".into(),
            items: vec![Item::json(
                json!({"message": case.input, "messages": [{"role": "user", "content": case.input}]}),
            )],
            trigger_node: None,
        };
        let answer = match runner.invoke(req).await {
            Ok(outcome) => outcome.response.first().map(|i| answer_text(&i.json)).unwrap_or_default(),
            Err(e) => {
                failures
                    .push(Failure { node_id: None, problem: format!("asked \"{}\", the run failed: {e}", case.input) });
                continue;
            }
        };
        match judge(models, case, &answer).await {
            Ok((verdict, usd)) => {
                cost += usd;
                if verdict.pass {
                    passed += 1;
                } else {
                    failures.push(Failure {
                        node_id: None,
                        problem: format!(
                            "asked \"{}\", a good answer {}, but it answered \"{answer}\" ({})",
                            case.input, case.expect, verdict.reason
                        ),
                    });
                }
            }
            Err(e) => tracing::warn!(flow_id, error = %e, "could not judge a test case"),
        }
    }
    (passed, failures, cost)
}

fn answer_text(json: &Json) -> String {
    let value = json.get("answer").or_else(|| json.get("message")).unwrap_or(json);
    value.as_str().map_or_else(|| value.to_string(), str::to_owned)
}

async fn judge(models: &dyn ModelGateway, case: &Case, answer: &str) -> Result<(Verdict, f64), ModelError> {
    let prompt = format!(
        "A user asked: {}\nA good answer {}\nThe app answered: {answer}\n\nDoes the answer do that? Reply with JSON: \
         {{\"pass\": true | false, \"reason\": \"one short sentence\"}}",
        case.input, case.expect
    );
    let schema = json!({"type": "object", "required": ["pass"],
        "properties": {"pass": {"type": "boolean"}, "reason": {"type": "string"}}});
    let req = ChatRequest {
        messages: vec![ChatMessage::user(prompt)],
        json_schema: Some(schema),
        temperature: Some(0.0),
        ..Default::default()
    };
    let resp = models.chat(ModelRole::Fast, req).await?;
    let verdict =
        parse(&resp.content).unwrap_or(Verdict { pass: false, reason: "the judge's reply was unreadable".into() });
    Ok((verdict, resp.usage.cost_usd))
}

fn parse<T: for<'de> Deserialize<'de>>(content: &str) -> Option<T> {
    let body = match (content.find('{'), content.rfind('}')) {
        (Some(a), Some(b)) if a < b => &content[a..=b],
        _ => content,
    };
    serde_json::from_str(body).ok()
}
