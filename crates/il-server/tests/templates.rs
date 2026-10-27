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
