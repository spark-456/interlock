//! `trigger.schedule` — Schedule.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B05 implements it.

use il_sdk::prelude::*;

/// Schedule node.
#[derive(Debug, Default)]
pub struct TriggerSchedule;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.schedule",
        Category::Trigger,
        "Schedule",
        "Starts the flow on a cron schedule in a timezone, e.g. every Monday at 9:00.",
    )
    .icon("clock")
    .trigger()
    .settings_schema(json!({
        "type": "object",
        "required": [
            "cron"
        ],
        "properties": {
            "cron": {
                "type": "string",
                "title": "Cron",
                "description": "Five-field cron, e.g. `0 9 * * MON`."
            },
            "timezone": {
                "type": "string",
                "title": "Timezone",
                "default": "UTC"
            }
        }
    }))
    .example(
        "Every Monday 9:00",
        json!({
            "cron": "0 9 * * MON",
            "timezone": "Asia/Kolkata"
        }),
    )
});

#[async_trait]
impl NodeExecutor for TriggerSchedule {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerSchedule)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::TriggerSchedule);
    }
}
