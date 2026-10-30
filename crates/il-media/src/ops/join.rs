//! Joins generated shots into one video for `ai.generate_video`.

use il_sdk::prelude::*;

/// Scales every clip to the aspect's frame and plays them back to back, without audio.
pub(crate) async fn join_clips(ctx: &NodeCtx, clips: &[FileRef], aspect: &str) -> Result<FileRef, NodeError> {
    let (width, height) = if aspect == "9:16" { (720, 1280) } else { (1280, 720) };
    let mut args: Vec<std::ffi::OsString> = Vec::new();
    let mut expected = 0.0;
    for clip in clips {
        let path = super::ffmpeg::input_path(ctx, &clip.id).await?;
        expected += super::ffmpeg::duration(&path).await?;
        args.extend(["-i".into(), path.into_os_string()]);
    }
    let scaled: String = (0..clips.len())
        .map(|i| {
            format!(
                "[{i}:v]scale={width}:{height}:force_original_aspect_ratio=decrease,\
                 pad={width}:{height}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=24[v{i}];"
            )
        })
        .collect();
    let inputs: String = (0..clips.len()).map(|i| format!("[v{i}]")).collect();
    let target = super::ffmpeg::output_path("mp4");
    args.extend([
        "-filter_complex".into(),
        format!("{scaled}{inputs}concat=n={}:v=1:a=0[v]", clips.len()).into(),
        "-map".into(),
        "[v]".into(),
        "-c:v".into(),
        "libx264".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-preset".into(),
        "veryfast".into(),
        "-movflags".into(),
        "+faststart".into(),
        target.as_os_str().into(),
    ]);
    super::ffmpeg::run(ctx, &args, Some(expected), None).await?;
    super::ffmpeg::save(ctx, &target, "generated.mp4", "video/mp4").await
}
