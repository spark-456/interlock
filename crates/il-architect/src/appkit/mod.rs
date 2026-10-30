//! The AI app kit (AR-9): checks that an app has safety rules, writes a test suite of knowledge
//! questions and tool tasks from its goal, runs each case through the flow, and judges the answers.

use il_core::{Flow, Item, Json};
use il_sdk::Services;
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelError, ModelGateway, ModelRole};
use il_sdk::services::runner::{FlowRunner, StartRun};
use serde::Deserialize;
use serde_json::json;

use crate::catalog::Catalog;
use crate::testfix::Failure;

const GUARDRAILS: &str = "ai.guardrails";
/// Cases in a generated suite.
const MAX_CASES: usize = 5;
/// Passages from the app's knowledge shown to the judge for a knowledge question.
const PASSAGES: usize = 4;
/// Characters of each passage shown to the judge.
const PASSAGE_CHARS: usize = 600;

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
    /// With passages: whether they hold what a good answer needs.
    #[serde(default)]
    covered: Option<bool>,
    /// With passages: whether the answer says it couldn't find it, without making anything up.
    #[serde(default)]
    declines: Option<bool>,
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
    // An app without tools can only fail a task that needs one, however good it is.
    let has_tools = flow.nodes.iter().any(|n| {
        n.node_type == "ai.agent"
            && (n.settings["tools"].as_array().is_some_and(|t| !t.is_empty()) || n.settings["apiTools"].is_object())
    });
    let (kinds, wanted) = if has_tools {
        (json!(["knowledge", "tool"]), "questions its knowledge should answer and tasks that need its tools")
    } else {
        (json!(["knowledge"]), "questions its knowledge should answer (it has no tools, so no tasks that need one)")
    };
    let text = json!({"type": "string"});
    let schema = json!({"type": "object", "required": ["cases"], "properties": {"cases": {"type": "array",
        "items": {"type": "object", "required": ["kind", "input", "expect"], "properties": {
            "kind": {"type": "string", "enum": kinds}, "input": text, "expect": text}}}}});
    let prompt = format!(
        "{brief}\n\nThe AI app \"{}\" was built for this. Write {MAX_CASES} test cases a real user would send: \
         {wanted}. For each, say in one sentence what a good answer does. Don't invent account details (emails, ids, \
         orders) and expect them to be found: a tool task should expect the app to try the lookup and either use what \
         it finds or say honestly it found nothing. Reply with JSON: {{\"cases\": [{{\"kind\": \"knowledge\" | \
         \"tool\", \"input\": \"…\", \"expect\": \"…\"}}]}}",
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
    let cases = cases.into_iter().filter(|c| has_tools || c.kind != "tool").take(MAX_CASES).collect();
    Ok((cases, resp.usage.cost_usd))
}

/// Sends each case to the app and judges the answer with the Fast model. Returns how many
/// passed, a failure per failed case, and the judging cost.
pub async fn run(
    services: &Services,
    runner: &dyn FlowRunner,
    flow_id: &str,
    cases: &[Case],
) -> (u32, Vec<Failure>, f64) {
    let models = services.models.as_ref();
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
            only: None,
        };
        let outcome = runner.invoke(req).await;
        let passages = match &outcome {
            Ok(outcome) if case.kind == "knowledge" => passages(services, &outcome.run.id, &case.input).await,
            _ => None,
        };
        let answer = match outcome {
            Ok(outcome) if outcome.response.is_empty() && outcome.run.error.is_some() => {
                let error = outcome.run.error.unwrap_or_default();
                failures.push(Failure {
                    node_id: None,
                    problem: format!("asked \"{}\", the run failed: {error}", case.input),
                });
                continue;
            }
            Ok(outcome) => outcome.response.first().map(|i| answer_text(&i.json)).unwrap_or_default(),
            Err(e) => {
                failures
                    .push(Failure { node_id: None, problem: format!("asked \"{}\", the run failed: {e}", case.input) });
                continue;
            }
        };
        match judge(models, case, &answer, passages.as_deref()).await {
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

/// What the app's knowledge bases hold on `question`, searched in the bases the run's knowledge
/// steps used; `None` when the run used none. Generated questions can ask what the documents don't
/// cover, and an app that says so must not fail for it.
async fn passages(services: &Services, run_id: &str, question: &str) -> Option<String> {
    let node_runs = services.store.node_runs(run_id).await.unwrap_or_default();
    let bases: Vec<&str> = node_runs
        .iter()
        .flat_map(|n| n.output.get("kb").into_iter().flatten())
        .filter_map(|item| item.json["knowledgeBase"].as_str())
        .collect();
    if bases.is_empty() {
        return None;
    }
    let mut hits = Vec::new();
    for base in bases {
        hits.extend(services.knowledge.search(base, question, PASSAGES).await.unwrap_or_default());
    }
    hits.sort_by(|a, b| b.score.total_cmp(&a.score));
    let lines: Vec<String> = hits
        .iter()
        .take(PASSAGES)
        .map(|h| format!("- {}: {}", h.title, h.text.chars().take(PASSAGE_CHARS).collect::<String>()))
        .collect();
    Some(if lines.is_empty() { "(nothing)".into() } else { lines.join("\n") })
}

/// Judges `answer` with the Fast model. Given passages, the model says whether they cover the question
/// and whether the answer declines honestly, and an uncovered question passes on an honest decline:
/// asked only "does it do what a good answer does", models fail every honest "I couldn't find that".
async fn judge(
    models: &dyn ModelGateway,
    case: &Case,
    answer: &str,
    passages: Option<&str>,
) -> Result<(Verdict, f64), ModelError> {
    let flag = json!({"type": "boolean"});
    let (evidence, fields, schema) = match passages {
        Some(p) => (
            format!("\nWhat the app's documents say on this:\n{p}"),
            "\"covered\": do those passages hold what that good answer needs?, \"declines\": does the app say it \
             couldn't find it, without making anything up?, ",
            json!({"type": "object", "required": ["covered", "declines", "pass"], "properties": {
                "covered": flag, "declines": flag, "pass": flag, "reason": {"type": "string"}}}),
        ),
        None => (
            String::new(),
            "",
            json!({"type": "object", "required": ["pass"],
            "properties": {"pass": flag, "reason": {"type": "string"}}}),
        ),
    };
    let prompt = format!(
        "A user asked: {}\nA good answer {}{evidence}\nThe app answered: {answer}\n\nReply with JSON: {{{fields}\
         \"pass\": does the answer do what a good answer does?, \"reason\": one short sentence}}",
        case.input, case.expect
    );
    let req = ChatRequest {
        messages: vec![ChatMessage::user(prompt)],
        json_schema: Some(schema),
        temperature: Some(0.0),
        ..Default::default()
    };
    let resp = models.chat(ModelRole::Fast, req).await?;
    let mut verdict = parse(&resp.content).unwrap_or(Verdict {
        pass: false,
        reason: "the judge's reply was unreadable".into(),
        covered: None,
        declines: None,
    });
    if verdict.covered == Some(false) {
        verdict.pass = verdict.declines == Some(true);
        verdict.reason = format!("the documents don't cover it; {}", verdict.reason);
    }
    Ok((verdict, resp.usage.cost_usd))
}

fn parse<T: for<'de> Deserialize<'de>>(content: &str) -> Option<T> {
    let body = match (content.find('{'), content.rfind('}')) {
        (Some(a), Some(b)) if a < b => &content[a..=b],
        _ => content,
    };
    serde_json::from_str(body).ok()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use il_core::{ApprovalDecision, Item, NodeRun, RunStatus};
    use il_sdk::services::knowledge::SearchHit;
    use il_sdk::services::models::ChatRole;
    use il_sdk::services::runner::{FlowRunner, InvokeOutcome, RunnerError, StartRun};
    use il_sdk::services::store::Store;
    use il_sdk::testkit::{FakeRunner, FakeServices};
    use serde_json::json;

    use super::{Case, run, suite};

    /// Answers every case "not found" after a knowledge step that used base `kb_1`.
    struct KbRunner(Arc<dyn Store>);

    #[async_trait]
    impl FlowRunner for KbRunner {
        async fn start(&self, _req: StartRun) -> Result<String, RunnerError> {
            Ok("run_kb".into())
        }
        async fn invoke(&self, req: StartRun) -> Result<InvokeOutcome, RunnerError> {
            let mut run = FakeRunner::default().invoke(req).await?.run;
            run.id = "run_kb".into();
            let output = [("kb".to_string(), vec![Item::json(json!({"knowledgeBase": "kb_1"}))])].into();
            let node_run = NodeRun {
                run_id: run.id.clone(),
                node_id: "n_kb".into(),
                status: RunStatus::Succeeded,
                attempt: 1,
                input: vec![],
                output,
                error: None,
                ms: 1,
                cost_usd: 0.0,
            };
            self.0.put_node_run(&node_run).await.unwrap();
            Ok(InvokeOutcome { run, response: vec![Item::json(json!({"answer": "Our documents don't say."}))] })
        }
        async fn decide(&self, _id: &str, _d: ApprovalDecision, _by: Option<String>) -> Result<(), RunnerError> {
            Ok(())
        }
        async fn cancel(&self, _run_id: &str) -> Result<(), RunnerError> {
            Ok(())
        }
        async fn replay(&self, _id: &str) -> Result<String, RunnerError> {
            Ok("run_replay".into())
        }
    }

    #[tokio::test]
    async fn q29_the_judge_sees_what_the_documents_say() {
        let fakes = FakeServices::default();
        fakes.knowledge.hits.lock().unwrap().push(SearchHit {
            text: "Refunds are issued within 30 days.".into(),
            score: 0.8,
            source_id: "src_1".into(),
            title: "Refunds".into(),
            section: None,
            url: None,
        });
        fakes.models.push_json(json!({"covered": false, "declines": true, "pass": false}));
        fakes.models.push_json(json!({"pass": true}));
        let case = |kind: &str| Case { kind: kind.into(), input: "Is there a free trial?".into(), expect: "x".into() };
        let runner = KbRunner(fakes.store.clone());
        let (passed, failures, _) = run(&fakes.services(), &runner, "flw_1", &[case("knowledge"), case("tool")]).await;
        assert_eq!((passed, failures.len()), (2, 0));
        let prompts: Vec<String> = fakes
            .models
            .calls()
            .iter()
            .map(|(_, r)| r.messages.iter().filter(|m| m.role == ChatRole::User).map(|m| m.content.clone()).collect())
            .collect();
        assert!(
            prompts[0].contains("- Refunds: Refunds are issued within 30 days.") && prompts[0].contains("\"covered\"")
        );
        assert!(!prompts[1].contains("documents say"), "a tool task is judged on its own");
    }

    #[tokio::test]
    async fn q29_an_app_without_tools_gets_no_tool_tasks() {
        let fakes = FakeServices::default();
        fakes.models.push_json(json!({"cases": [
            {"kind": "knowledge", "input": "What is the refund window?", "expect": "says 30 days"},
            {"kind": "tool", "input": "Which card do you have on file?", "expect": "looks it up"}]}));
        let flow: il_core::Flow = serde_json::from_value(json!({"id": "flw_1", "workspaceId": "ws", "version": 1,
            "name": "Help", "kind": "app", "edges": [],
            "nodes": [{"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"knowledgeBases": ["n_kb"]}}]}))
        .unwrap();
        let (cases, _) = suite(fakes.models.as_ref(), &flow, "Goal: answer billing questions").await.unwrap();
        assert_eq!(cases.iter().map(|c| c.kind.as_str()).collect::<Vec<_>>(), ["knowledge"]);
        let (_, req) = &fakes.models.calls()[0];
        assert!(req.messages[0].content.contains("it has no tools"));
    }
}
