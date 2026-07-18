#![forbid(unsafe_code)]
//! Runnable local STE web gateway composition.

use std::{
    collections::BTreeMap,
    env, fs,
    net::SocketAddr,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use ste_query_plane::Scope;
use ste_ui_gateway::{AssetManifest, Role};
use ste_web_gateway::{
    BrowserSession, ProductionServices, SessionRecord, VerifiedAssets, WebConfig, router, serve,
};
use ste_workflows::{Authorization, AuthorizationDecision, InMemoryJournal, WorkflowRequest};

struct LocalPolicyAuthority {
    authorization_ref: String,
    purpose_ref: String,
}
impl Authorization for LocalPolicyAuthority {
    fn reauthorize(&self, request: &WorkflowRequest, now_ms: u64) -> AuthorizationDecision {
        if request.authorization_ref == self.authorization_ref
            && request.purpose_ref == self.purpose_ref
            && request.expires_at_ms > now_ms
            && !matches!(request.requester_role, ste_workflows::Role::Viewer)
        {
            AuthorizationDecision::Allow
        } else {
            AuthorizationDecision::Preempt("local policy denied".to_owned())
        }
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("required environment variable {name} is not set"))
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_millis() as u64)
}

fn load_assets(root: &Path) -> Result<VerifiedAssets, String> {
    let manifest_bytes = fs::read(root.join("asset-manifest.json"))
        .map_err(|error| format!("cannot read asset manifest: {error}"))?;
    let manifest: AssetManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("invalid asset manifest: {error}"))?;
    let mut files = BTreeMap::new();
    for entry in &manifest.assets {
        let path = root.join(&entry.path);
        let body = fs::read(&path)
            .map_err(|error| format!("cannot read verified asset {}: {error}", entry.path))?;
        files.insert(entry.path.clone(), body);
    }
    VerifiedAssets::new(&manifest, files)
        .map_err(|_| "UI distribution failed manifest verification".to_owned())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind: SocketAddr = env::var("STE_WEB_BIND")
        .unwrap_or_else(|_| "127.0.0.1:4173".to_owned())
        .parse()?;
    let origin = env::var("STE_WEB_ORIGIN").unwrap_or_else(|_| format!("http://{bind}"));
    let asset_root = env::var("STE_UI_DIST").unwrap_or_else(|_| "ui/dist".to_owned());
    let cookie = required("STE_SESSION_COOKIE")?;
    let csrf = required("STE_CSRF_TOKEN")?;
    if cookie.len() < 32 || csrf.len() < 16 {
        return Err("session cookie must be >=32 and CSRF token >=16 characters".into());
    }
    let authorization_ref = required("STE_AUTHORIZATION_REF")?;
    let purpose_ref = required("STE_PURPOSE_REF")?;
    let authority = LocalPolicyAuthority {
        authorization_ref: authorization_ref.clone(),
        purpose_ref: purpose_ref.clone(),
    };
    let services = Arc::new(ProductionServices::new(
        1024,
        InMemoryJournal::default(),
        authority,
    )?);
    services.register_session(
        cookie,
        SessionRecord {
            session: BrowserSession {
                id: format!("local-{}", now_ms()),
                role: Role::Operator,
                capabilities: vec![
                    "query.live".to_owned(),
                    "query.history".to_owned(),
                    "workflow.create".to_owned(),
                    "workflow.apply".to_owned(),
                ],
                csrf,
            },
            expires_at_ms: now_ms().saturating_add(900_000),
            query_scope: Scope::Operator,
            authorization_ref,
            purpose_ref,
        },
    )?;
    let config = WebConfig {
        bind,
        host: bind.to_string(),
        origin,
        max_body_bytes: 1_048_576,
        max_connections: 128,
        max_streams: 32,
        request_timeout: Duration::from_secs(15),
    };
    let app = router(
        config.clone(),
        services,
        load_assets(Path::new(&asset_root))?,
    )?;
    eprintln!("STE web gateway listening on {}", config.bind);
    serve(config, app).await?;
    Ok(())
}
