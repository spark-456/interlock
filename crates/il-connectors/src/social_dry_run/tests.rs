use il_sdk::prelude::*;
use il_sdk::services::FileStore;
use il_sdk::testkit::NodeHarness;

#[tokio::test]
async fn c03_instagram_and_tiktok_preview_video_without_posting() {
    let instagram = NodeHarness::new(super::instagram::AppInstagramPostReel);
    let file = instagram.fakes.files.put("clip.mp4", "video/mp4", "video".into()).await.unwrap();
    let mut item = Item::default();
    item.files.insert("clip_9x16".into(), file);
    let out = instagram
        .settings(json!({"video":"{{ item.files.clip_9x16 }}","caption":"New product"}))
        .input(vec![item])
        .run()
        .await
        .unwrap();
    assert_eq!(out.ports["out"][0].json["dryRun"]["status"], "dryRun");
    assert_eq!(out.ports["out"][0].json["dryRun"]["preview"]["text"], "New product");
    assert_eq!(out.ports["out"][0].json["dryRun"]["note"], "Needs platform approval to post");

    let tiktok = NodeHarness::new(super::tiktok::AppTiktokPost);
    let bad = tiktok.fakes.files.put("picture.png", "image/png", "pixels".into()).await.unwrap();
    let err = tiktok.settings(json!({"video":bad.id,"caption":"Product"})).run().await.unwrap_err();
    assert!(err.to_string().contains("MP4, WebM or MOV"));
}

#[tokio::test]
async fn c03_linkedin_and_x_validate_and_preview_text() {
    let linked_in = NodeHarness::new(super::linkedin::AppLinkedinPost)
        .settings(json!({"text":"Launch week","visibility":"connections"}))
        .run()
        .await
        .unwrap();
    assert_eq!(linked_in.ports["out"][0].json["dryRun"]["platform"], "linkedin");
    assert_eq!(linked_in.ports["out"][0].json["dryRun"]["preview"]["options"]["visibility"], "connections");
    let x = NodeHarness::new(super::x::AppXPost).settings(json!({"text":"Hello"})).run().await.unwrap();
    assert_eq!(x.ports["out"][0].json["dryRun"]["platform"], "x");
    let err = NodeHarness::new(super::x::AppXPost).settings(json!({"text":"  "})).run().await.unwrap_err();
    assert!(matches!(err, NodeError::InvalidSettings(_)));
}
