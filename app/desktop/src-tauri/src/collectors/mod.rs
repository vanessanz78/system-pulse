use crate::models::SystemSnapshot;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub fn collect_system_snapshot() -> Result<SystemSnapshot, String> {
    macos::collect_system_snapshot()
}

#[cfg(not(target_os = "macos"))]
pub fn collect_system_snapshot() -> Result<SystemSnapshot, String> {
    Err("System Pulse Milestone 2 collectors currently support macOS only.".to_string())
}
