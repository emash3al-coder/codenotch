//! Ollama's local-only monitor. /api/ps is lightweight and reports models currently loaded.

use crate::activity::Activity;
use crate::usage::UsageSnapshot;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

pub fn executable() -> Option<std::path::PathBuf> {
    let mut candidates = Vec::new();
    if let Some(local) = dirs::data_local_dir() {
        candidates.push(local.join("Programs").join("Ollama").join("ollama.exe"));
        candidates.push(local.join("Ollama").join("ollama.exe"));
    }
    if let Some(programs) = std::env::var_os("ProgramFiles") {
        candidates.push(std::path::PathBuf::from(programs).join("Ollama").join("ollama.exe"));
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|p| p.join("ollama.exe")));
    }
    candidates.into_iter().find(|p| p.is_file())
}

pub fn present() -> bool {
    executable().is_some()
}

fn loaded_models() -> Vec<String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_millis(500)).build();
    let Ok(response) = agent.get("http://127.0.0.1:11434/api/ps").call() else { return Vec::new() };
    let Ok(value) = response.into_json::<serde_json::Value>() else { return Vec::new() };
    value.get("models").and_then(|v| v.as_array()).into_iter().flatten()
        .filter_map(|model| model.get("name").and_then(|v| v.as_str()).map(String::from))
        .collect()
}

pub fn probe() -> (UsageSnapshot, Vec<Activity>) {
    if !present() {
        return (UsageSnapshot { status: "absent".into(), ..Default::default() }, Vec::new());
    }
    let at = now_ms();
    let models = loaded_models();
    if models.is_empty() {
        return (
            UsageSnapshot { status: "ok".into(), fetched_at: at, note: "Idle · Local / Free".into(), ..Default::default() },
            Vec::new(),
        );
    }
    let detail = format!("Running · Local / Free · {}", models.join(", "));
    let activities = models.into_iter().map(|name| Activity {
        provider: "ollama".into(),
        state: "busy".into(),
        name,
        detail: "Local / Free".into(),
        since: at,
    }).collect();
    (
        UsageSnapshot { status: "ok".into(), fetched_at: at, note: detail, ..Default::default() },
        activities,
    )
}
