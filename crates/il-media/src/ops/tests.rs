use bytes::Bytes;
use il_sdk::prelude::*;
use il_sdk::services::FileStore;
use il_sdk::testkit::NodeHarness;
use tokio::process::Command;

async fn tiny_video() -> Option<Vec<u8>> {
    if Command::new("ffmpeg").arg("-version").output().await.is_err() {
        eprintln!("M01 media fixture test skipped: ffmpeg is unavailable");
        return None;
    }
    let path = super::ffmpeg::output_path("mp4");
    let output = Command::new("ffmpeg")
        .args(["-nostdin", "-y", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc2=size=128x72:rate=5"])
        .args([
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=44100",
            "-t",
            "1",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
        ])
        .arg(&path)
        .output()
        .await
        .expect("ffmpeg fixture process");
    assert!(output.status.success(), "ffmpeg fixture: {}", String::from_utf8_lossy(&output.stderr));
    let bytes = tokio::fs::read(&path).await.expect("fixture bytes");
    tokio::fs::remove_file(&path).await.expect("remove fixture");
    Some(bytes)
}

async fn run<N: NodeExecutor>(
    node: N,
    mut settings: Value,
    fixture: &[u8],
    file_key: &str,
) -> (NodeOutput, NodeHarness<N>) {
    let type_id = node.manifest().type_id.clone();
    let harness = NodeHarness::new(node);
    let file = harness.fakes.files.put("source.mp4", "video/mp4", Bytes::copy_from_slice(fixture)).await.unwrap();
    settings[file_key] = json!(file.id);
    if file_key == "video" && settings.get("audio").is_some() {
        settings["audio"] = json!(file.id);
    }
    let harness = harness.settings(settings);
    let output = harness.run().await.unwrap_or_else(|e| panic!("{type_id} should process fixture: {e}"));
    for item in output.ports.get("out").into_iter().flatten() {
        for reference in item.files.values() {
            let (_, bytes) = harness.fakes.files.get(&reference.id).await.expect("output stored");
            assert!(!bytes.is_empty(), "empty output file");
        }
    }
    (output, harness)
}

#[tokio::test]
async fn m01_ffmpeg_ops_process_real_fixture() {
    let Some(video) = tiny_video().await else { return };

    let (cut, progress) =
        run(super::cut_clips::MediaCutClips, json!({"count":2,"durationSec":3,"strategy":"even"}), &video, "video")
            .await;
    assert!(cut.ports["out"][0].files.contains_key("clip_1"));
    assert!(cut.ports["out"][0].files.contains_key("clip_2"));
    assert!(!progress.progress.events.lock().unwrap().is_empty(), "ffmpeg progress events");

    let (resized, _) = run(super::resize::MediaResize, json!({"aspects":["1:1"],"fit":"pad"}), &video, "video").await;
    assert!(resized.ports["out"][0].files.contains_key("clip_1x1"));

    let (thumbnail, _) = run(super::thumbnail::MediaThumbnail, json!({"atSec":0,"width":128}), &video, "video").await;
    assert_eq!(thumbnail.ports["out"][0].files["thumbnail"].mime, "image/jpeg");

    let (merged, _) =
        run(super::merge_av::MediaMergeAv, json!({"audio":"same","mode":"mix","volume":0.5}), &video, "video").await;
    assert!(merged.ports["out"][0].files.contains_key("merged"));

    let (captioned, _) =
        run(super::captions::MediaCaptions, json!({"text":"Hello","position":"bottom"}), &video, "video").await;
    assert!(captioned.ports["out"][0].files.contains_key("captioned"));
    let (srt, _) = run(
        super::captions::MediaCaptions,
        json!({"text":"1\n00:00:00,000 --> 00:00:01,000\nHello\n"}),
        &video,
        "video",
    )
    .await;
    assert!(srt.ports["out"][0].files.contains_key("captioned"));

    let (converted, _) = run(super::convert::MediaConvert, json!({"format":"mp3"}), &video, "file").await;
    assert_eq!(converted.ports["out"][0].files["converted"].mime, "audio/mpeg");

    let harness = NodeHarness::new(super::thumbnail::MediaThumbnail);
    let source = harness.fakes.files.put("source.mp4", "video/mp4", Bytes::copy_from_slice(&video)).await.unwrap();
    let mut item = Item::default();
    item.files.insert("video".into(), source);
    let output = harness
        .settings(json!({"video":"{{ item.files.video }}","atSec":0,"width":128}))
        .input(vec![item])
        .run()
        .await
        .expect("default file expression falls back to item.files");
    assert!(output.ports["out"][0].files.contains_key("thumbnail"));
}

#[tokio::test]
async fn q33_generated_shots_join_into_one_video() {
    let Some(video) = tiny_video().await else { return };
    let harness = NodeHarness::new(super::thumbnail::MediaThumbnail);
    let mut clips = Vec::new();
    for _ in 0..2 {
        clips.push(harness.fakes.files.put("shot.mp4", "video/mp4", Bytes::copy_from_slice(&video)).await.unwrap());
    }
    let ctx = harness.ctx();
    let joined = super::join::join_clips(&ctx, &clips, "9:16").await.expect("joined");
    let path = super::ffmpeg::input_path(&ctx, &joined.id).await.unwrap();
    let seconds = super::ffmpeg::duration(&path).await.unwrap();
    assert!((1.8..2.5).contains(&seconds), "two 1 s shots joined: {seconds}");
}
