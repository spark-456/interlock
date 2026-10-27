//! Records beyond the core `Store` trait: identities, keys, triggers and builds.

use chrono::{DateTime, Utc};
use il_core::{Flow, Run};
use il_sdk::services::store::{Credential, StoreError, StoreResult};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    db::{SqliteStore, backend},
    secrets,
};

/// Workspace identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
/// User identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
/// API key metadata; the raw key is never persisted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
/// A cron trigger definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleRecord {
    pub id: String,
    pub flow_id: String,
    pub cron: String,
    pub timezone: String,
    pub enabled: bool,
}
/// A webhook trigger definition; its token is stored only as a hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebhookRecord {
    pub id: String,
    pub flow_id: String,
    pub enabled: bool,
}
/// Architect build state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildRecord {
    pub id: String,
    pub workspace_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub body: Value,
}

fn hash(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}
fn stamp(value: &str) -> StoreResult<DateTime<Utc>> {
    value.parse().map_err(backend)
}

impl SqliteStore {
    /// Inserts or updates a workspace.
    pub async fn put_workspace(&self, record: &WorkspaceRecord) -> StoreResult<()> {
        let r = record.clone();
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO ws_workspaces (id,name,created_at) VALUES (?1,?2,?3)
                       ON CONFLICT(id) DO UPDATE SET name=excluded.name",
                params![r.id, r.name, r.created_at.to_rfc3339()],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Returns a workspace by ID.
    pub async fn get_workspace(&self, id: &str) -> StoreResult<Option<WorkspaceRecord>> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            let row: Option<(String, String, String)> = c
                .query_row("SELECT id,name,created_at FROM ws_workspaces WHERE id=?1", [&id], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })
                .optional()
                .map_err(backend)?;
            row.map(|(id, name, created_at)| Ok(WorkspaceRecord { id, name, created_at: stamp(&created_at)? }))
                .transpose()
        })
        .await
    }

    /// Inserts or updates a user.
    pub async fn put_user(&self, record: &UserRecord) -> StoreResult<()> {
        let r = record.clone();
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO ws_users (id,email,name,created_at) VALUES (?1,?2,?3,?4)
                       ON CONFLICT(id) DO UPDATE SET email=excluded.email, name=excluded.name",
                params![r.id, r.email, r.name, r.created_at.to_rfc3339()],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Returns a user by ID.
    pub async fn get_user(&self, id: &str) -> StoreResult<Option<UserRecord>> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            let row: Option<(String, String, String, String)> = c
                .query_row("SELECT id,email,name,created_at FROM ws_users WHERE id=?1", [&id], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                })
                .optional()
                .map_err(backend)?;
            row.map(|(id, email, name, created_at)| Ok(UserRecord { id, email, name, created_at: stamp(&created_at)? }))
                .transpose()
        })
        .await
    }

    /// Sets a user's role in a workspace.
    pub async fn put_member(&self, workspace_id: &str, user_id: &str, role: &str) -> StoreResult<()> {
        let (ws, user, role) = (workspace_id.to_owned(), user_id.to_owned(), role.to_owned());
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO ws_members (workspace_id,user_id,role) VALUES (?1,?2,?3)
                       ON CONFLICT(workspace_id,user_id) DO UPDATE SET role=excluded.role",
                params![ws, user, role],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Returns a user's role in a workspace, if any.
    pub async fn member_role(&self, workspace_id: &str, user_id: &str) -> StoreResult<Option<String>> {
        let (ws, user) = (workspace_id.to_owned(), user_id.to_owned());
        self.with_conn(move |c| {
            c.query_row("SELECT role FROM ws_members WHERE workspace_id=?1 AND user_id=?2", params![ws, user], |r| {
                r.get(0)
            })
            .optional()
            .map_err(backend)
        })
        .await
    }

    /// Creates a key from a one-time raw secret. Only its SHA-256 digest is saved.
    pub async fn create_api_key(&self, id: &str, workspace_id: &str, name: &str, raw_key: &str) -> StoreResult<()> {
        let (id, ws, name, digest) = (id.to_owned(), workspace_id.to_owned(), name.to_owned(), hash(raw_key));
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO ws_api_keys (id,workspace_id,name,key_hash,created_at) VALUES (?1,?2,?3,?4,?5)",
                params![id, ws, name, digest, Utc::now().to_rfc3339()],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Lists API key metadata without exposing key material.
    pub async fn list_api_keys(&self, workspace_id: &str) -> StoreResult<Vec<ApiKeyRecord>> {
        let ws = workspace_id.to_owned();
        self.with_conn(move |c| {
            let mut stmt = c
                .prepare(
                    "SELECT id,workspace_id,name,created_at,last_used_at,revoked_at
                FROM ws_api_keys WHERE workspace_id=?1 ORDER BY created_at DESC",
                )
                .map_err(backend)?;
            let rows = stmt
                .query_map([ws], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, Option<String>>(5)?,
                    ))
                })
                .map_err(backend)?;
            rows.map(|row| {
                let (id, workspace_id, name, created_at, last_used_at, revoked_at) = row.map_err(backend)?;
                Ok(ApiKeyRecord {
                    id,
                    workspace_id,
                    name,
                    created_at: stamp(&created_at)?,
                    last_used_at: last_used_at.as_deref().map(stamp).transpose()?,
                    revoked_at: revoked_at.as_deref().map(stamp).transpose()?,
                })
            })
            .collect()
        })
        .await
    }

    /// Validates a key, updates its last-use timestamp, and returns its workspace.
    pub async fn workspace_for_api_key(&self, raw_key: &str) -> StoreResult<Option<String>> {
        let digest = hash(raw_key);
        self.with_conn(move |c| {
            let tx = c.transaction().map_err(backend)?;
            let row: Option<(String, String)> = tx
                .query_row(
                    "SELECT id,workspace_id FROM ws_api_keys WHERE key_hash=?1 AND revoked_at IS NULL",
                    [&digest],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .map_err(backend)?;
            if let Some((id, _)) = &row {
                tx.execute("UPDATE ws_api_keys SET last_used_at=?2 WHERE id=?1", params![id, Utc::now().to_rfc3339()])
                    .map_err(backend)?;
            }
            tx.commit().map_err(backend)?;
            Ok(row.map(|(_, ws)| ws))
        })
        .await
    }

    /// Revokes an API key.
    pub async fn revoke_api_key(&self, id: &str) -> StoreResult<()> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            if c.execute("UPDATE ws_api_keys SET revoked_at=?2 WHERE id=?1", params![id, Utc::now().to_rfc3339()])
                .map_err(backend)?
                == 0
            {
                return Err(StoreError::NotFound(id));
            }
            Ok(())
        })
        .await
    }

    /// Stores a credential with authenticated encryption.
    pub async fn put_credential(&self, workspace_id: &str, credential: &Credential) -> StoreResult<()> {
        let key = self.key.ok_or_else(|| StoreError::Backend("IL_MASTER_KEY is required for credentials".into()))?;
        let ws = workspace_id.to_owned();
        let r = credential.clone();
        let secret = serde_json::to_vec(&r.secret).map_err(backend)?;
        let meta = serde_json::to_string(&r.meta).map_err(backend)?;
        let (nonce, ciphertext) = secrets::seal(&key, &ws, &r.id, &secret)?;
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO cn_credentials (id,workspace_id,app,nonce,ciphertext,meta) VALUES (?1,?2,?3,?4,?5,?6)
                       ON CONFLICT(id) DO UPDATE SET workspace_id=excluded.workspace_id, app=excluded.app,
                       nonce=excluded.nonce, ciphertext=excluded.ciphertext, meta=excluded.meta",
                params![r.id, ws, r.app, nonce, ciphertext, meta],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Stores or updates a schedule.
    pub async fn put_schedule(&self, record: &ScheduleRecord) -> StoreResult<()> {
        let r = record.clone();
        self.with_conn(move |c| {
            c.execute("INSERT INTO tr_schedules (id,flow_id,cron,timezone,enabled) VALUES (?1,?2,?3,?4,?5)
                       ON CONFLICT(id) DO UPDATE SET cron=excluded.cron, timezone=excluded.timezone, enabled=excluded.enabled",
                params![r.id,r.flow_id,r.cron,r.timezone,r.enabled]).map_err(backend)?;
            Ok(())
        }).await
    }

    /// Lists enabled schedules.
    pub async fn enabled_schedules(&self) -> StoreResult<Vec<ScheduleRecord>> {
        self.with_conn(move |c| {
            let mut stmt = c
                .prepare("SELECT id,flow_id,cron,timezone,enabled FROM tr_schedules WHERE enabled=1 ORDER BY id")
                .map_err(backend)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(ScheduleRecord {
                        id: r.get(0)?,
                        flow_id: r.get(1)?,
                        cron: r.get(2)?,
                        timezone: r.get(3)?,
                        enabled: r.get(4)?,
                    })
                })
                .map_err(backend)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(backend)
        })
        .await
    }

    /// Stores a webhook using only the hash of its token.
    pub async fn put_webhook(&self, id: &str, flow_id: &str, raw_token: &str, enabled: bool) -> StoreResult<()> {
        let (id, flow_id, digest) = (id.to_owned(), flow_id.to_owned(), hash(raw_token));
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO tr_webhooks (id,flow_id,token_hash,enabled) VALUES (?1,?2,?3,?4)
                       ON CONFLICT(id) DO UPDATE SET token_hash=excluded.token_hash, enabled=excluded.enabled",
                params![id, flow_id, digest, enabled],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Resolves an enabled webhook from its one-time token.
    pub async fn webhook_for_token(&self, raw_token: &str) -> StoreResult<Option<WebhookRecord>> {
        let digest = hash(raw_token);
        self.with_conn(move |c| {
            c.query_row("SELECT id,flow_id,enabled FROM tr_webhooks WHERE token_hash=?1 AND enabled=1", [digest], |r| {
                Ok(WebhookRecord { id: r.get(0)?, flow_id: r.get(1)?, enabled: r.get(2)? })
            })
            .optional()
            .map_err(backend)
        })
        .await
    }

    /// Saves a build state snapshot.
    pub async fn put_build(&self, record: &BuildRecord) -> StoreResult<()> {
        let r = record.clone();
        self.with_conn(move |c| {
            c.execute(
                "INSERT INTO bd_builds (id,workspace_id,status,created_at,body) VALUES (?1,?2,?3,?4,?5)
                       ON CONFLICT(id) DO UPDATE SET status=excluded.status, body=excluded.body",
                params![
                    r.id,
                    r.workspace_id,
                    r.status,
                    r.created_at.to_rfc3339(),
                    serde_json::to_string(&r.body).map_err(backend)?
                ],
            )
            .map_err(backend)?;
            Ok(())
        })
        .await
    }

    /// Returns a saved build state snapshot.
    pub async fn get_build(&self, id: &str) -> StoreResult<Option<BuildRecord>> {
        let id = id.to_owned();
        self.with_conn(move |c| {
            let row: Option<(String, String, String, String, String)> = c
                .query_row("SELECT id,workspace_id,status,created_at,body FROM bd_builds WHERE id=?1", [&id], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                })
                .optional()
                .map_err(backend)?;
            row.map(|(id, workspace_id, status, created_at, body)| {
                Ok(BuildRecord {
                    id,
                    workspace_id,
                    status,
                    created_at: stamp(&created_at)?,
                    body: serde_json::from_str(&body).map_err(backend)?,
                })
            })
            .transpose()
        })
        .await
    }

    /// Returns active flows for trigger restoration after restart.
    pub async fn active_flows(&self) -> StoreResult<Vec<Flow>> {
        self.with_conn(move |c| {
            let mut stmt = c.prepare("SELECT body FROM fl_flows WHERE active=1 ORDER BY id").map_err(backend)?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(backend)?;
            rows.map(|row| serde_json::from_str(&row.map_err(backend)?).map_err(backend)).collect()
        })
        .await
    }

    /// Returns runs that can be recovered after restart.
    pub async fn recoverable_runs(&self) -> StoreResult<Vec<Run>> {
        self.with_conn(move |c| {
            let mut stmt = c
                .prepare("SELECT body FROM rn_runs WHERE status IN ('Running','Waiting','Queued') ORDER BY started_at")
                .map_err(backend)?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(backend)?;
            rows.map(|row| serde_json::from_str(&row.map_err(backend)?).map_err(backend)).collect()
        })
        .await
    }
}
