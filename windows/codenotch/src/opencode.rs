//! OpenCode installation and process presence. OpenCode does not expose a local quota source,
//! so this adapter deliberately reports status only.

use crate::activity::Activity;
use crate::usage::UsageSnapshot;

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

pub fn executable() -> Option<std::path::PathBuf> {
    if let Some(data) = dirs::data_dir() {
        let roaming = data.join("npm").join("opencode.cmd");
        if roaming.is_file() {
            return Some(roaming);
        }
    }
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|p| p.join("opencode.cmd"))
            .find(|p| p.is_file())
    })
}

pub fn present() -> bool {
    executable().is_some()
}

#[cfg(windows)]
fn running() -> bool {
    let maps = crate::focus::proc_maps();
    maps.name.values().any(|name| name == "opencode.exe" || name == "opencode")
}

#[cfg(not(windows))]
fn running() -> bool {
    false
}

pub fn probe() -> (UsageSnapshot, Vec<Activity>) {
    if !present() {
        return (UsageSnapshot { status: "absent".into(), ..Default::default() }, Vec::new());
    }
    let at = now_ms();
    if running() {
        let activity = Activity {
            provider: "opencode".into(),
            state: "busy".into(),
            name: "OpenCode".into(),
            detail: "Running".into(),
            since: at,
        };
        return (
            UsageSnapshot {
                status: "ok".into(),
                fetched_at: at,
                note: "Running · model not exposed by the local OpenCode process".into(),
                ..Default::default()
            },
            vec![activity],
        );
    }
    (
        UsageSnapshot {
            status: "ok".into(),
            fetched_at: at,
            note: "Idle · OpenCode installed".into(),
            ..Default::default()
        },
        Vec::new(),
    )
}
