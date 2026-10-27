//! Shared child-process and file-store helpers for ffmpeg nodes.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use bytes::Bytes;
use il_sdk::prelude::*;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};

pub(super) fn file_id(ctx: &NodeCtx, input: &NodeInput, key: &str) -> Result<String, NodeError> {
    ctx.settings
        .get(key)
        .and_then(|value| value.as_str().or_else(|| value["id"].as_str()))
        .filter(|id| !id.is_empty())
        .or_else(|| input.main().first().and_then(|item| item.files.get(key).map(|reference| reference.id.as_str())))
        .map(str::to_owned)
        .ok_or_else(|| NodeError::Failed(format!("missing {key} file")))
}

pub(super) async fn input_path(ctx: &NodeCtx, id: &str) -> Result<PathBuf, NodeError> {
    ctx.files().local_path(id).await.map_err(|e| NodeError::Failed(e.to_string()))
}

pub(super) fn output_path(extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}.{}", il_sdk::id("media"), extension))
}

pub(super) async fn save(ctx: &NodeCtx, path: &Path, name: &str, mime: &str) -> Result<FileRef, NodeError> {
    let bytes = tokio::fs::read(path).await.map_err(|e| NodeError::Failed(e.to_string()))?;
    let _ = tokio::fs::remove_file(path).await;
    ctx.files().put(name, mime, Bytes::from(bytes)).await.map_err(|e| NodeError::Failed(e.to_string()))
}

pub(super) fn with_file(input: NodeInput, key: &str, file: FileRef) -> NodeOutput {
    let mut item = input.main().first().cloned().unwrap_or_default();
    item.files.insert(key.into(), file);
    NodeOutput::out(vec![item])
}

pub(super) async fn duration(path: &Path) -> Result<f64, NodeError> {
    let output = timeout(
        Duration::from_secs(15),
        Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1"])
            .arg(path)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| NodeError::Timeout)?
    .map_err(|e| NodeError::Failed(format!("ffprobe unavailable: {e}")))?;
    if !output.status.success() {
        return Err(NodeError::Failed("ffprobe could not read the media".into()));
    }
    let value = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|_| NodeError::Failed("media duration unavailable".into()))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(NodeError::Failed("invalid media duration".into()));
    }
    Ok(value)
}

pub(super) async fn run(
    ctx: &NodeCtx,
    args: &[OsString],
    expected_sec: Option<f64>,
    cwd: Option<&Path>,
) -> Result<(), NodeError> {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-nostdin", "-y", "-hide_banner", "-loglevel", "error", "-progress", "pipe:1"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let mut child = command.spawn().map_err(|e| NodeError::Failed(format!("ffmpeg unavailable: {e}")))?;
    let stdout = child.stdout.take().ok_or_else(|| NodeError::Failed("ffmpeg progress unavailable".into()))?;
    let progress = ctx.progress.clone();
    let run = ctx.run.clone();
    let progress_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let (Some(duration), Some(us)) =
                (expected_sec, line.strip_prefix("out_time_us=").and_then(|v| v.parse::<f64>().ok()))
            {
                let pct = ((us / 1_000_000.0) / duration * 100.0).clamp(0.0, 99.0) as f32;
                progress.progress(&run, pct, "processing media");
            }
        }
    });
    let status = tokio::select! {
        result = timeout(Duration::from_secs(120), child.wait()) => match result {
            Ok(Ok(status)) => status,
            Ok(Err(_)) => { let _ = child.kill().await; return Err(NodeError::Failed("ffmpeg process failed".into())); }
            Err(_) => { let _ = child.kill().await; return Err(NodeError::Timeout); }
        },
        _ = ctx.cancel.cancelled() => { let _ = child.kill().await; return Err(NodeError::Cancelled); }
    };
    let _ = progress_task.await;
    if !status.success() {
        return Err(NodeError::Failed(format!("ffmpeg exited with {}", status.code().unwrap_or(-1))));
    }
    Ok(())
}
