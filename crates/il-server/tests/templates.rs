//! Every checked-in template (`fixtures/templates/*.json`) is a valid flow for the registered catalog.

use il_architect::catalog::Catalog;
use il_architect::validate::validate;
use il_core::Flow;

#[test]
fn every_template_validates_against_the_catalog() {
    il_server::wiring::link_all();
    let catalog = Catalog::registered();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/templates");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let flow: Flow = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(validate(&flow, &catalog), vec![], "{}", path.display());
        assert!(flow.meta.goal.as_deref().is_some_and(|g| !g.is_empty()), "{} needs a goal", path.display());
        count += 1;
    }
    assert_eq!(count, 10, "the gallery has ten templates (MK-2)");
}

#[test]
fn recorded_runs_belong_to_their_templates() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let mut count = 0;
    for entry in std::fs::read_dir(root.join("recorded")).unwrap() {
        let path = entry.unwrap().path();
        let events: Vec<il_core::events::RunEvent> = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let template: Flow =
            serde_json::from_slice(&std::fs::read(root.join("templates").join(path.file_name().unwrap())).unwrap())
                .unwrap();
        for e in &events {
            if let Some(node) = &e.node_id {
                assert!(template.node(node).is_some(), "{}: no step {node}", path.display());
            }
        }
        assert!(events.windows(2).all(|w| w[0].seq < w[1].seq && w[0].ts <= w[1].ts), "{}", path.display());
        let last = &events.last().unwrap().kind;
        assert!(matches!(last, il_core::events::RunEventKind::RunSucceeded { .. }), "{}", path.display());
        count += 1;
    }
    assert_eq!(count, 2, "F-A and F-C have recorded runs (MK-3)");
}
