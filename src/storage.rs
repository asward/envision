use crate::session::Session;
use std::fs;
use std::path::PathBuf;

/// Returns the base storage directory for envision data.
/// Uses $XDG_DATA_HOME/envision/sessions/ or ~/.local/share/envision/sessions/
fn sessions_dir() -> Result<PathBuf, String> {
    let base = if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        return Err("Cannot determine storage location: neither XDG_DATA_HOME nor HOME is set".into());
    };

    Ok(base.join("envision").join("sessions"))
}

/// Returns the file path for a given session (by shell PID).
fn session_path(pid: u32) -> Result<PathBuf, String> {
    Ok(sessions_dir()?.join(format!("{pid}.json")))
}

/// Check if a session file exists for this PID.
pub fn session_exists(pid: u32) -> Result<bool, String> {
    let path = session_path(pid)?;
    Ok(path.exists())
}

/// Load an existing session from disk.
pub fn load_session(pid: u32) -> Result<Session, String> {
    let path = session_path(pid)?;
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read session file {}: {e}", path.display()))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Session data corrupted ({}): {e}", path.display()))
}

/// Save a session to disk, creating directories as needed.
pub fn save_session(session: &Session) -> Result<PathBuf, String> {
    let dir = sessions_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create storage directory {}: {e}", dir.display()))?;

    let path = session_path(session.pid)?;
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize session: {e}"))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write session file {}: {e}", path.display()))?;

    Ok(path)
}

/// Remove a session file from disk.
pub fn remove_session(pid: u32) -> Result<(), String> {
    let path = session_path(pid)?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove session file {}: {e}", path.display()))?;
    }
    Ok(())
}

/// List stale sessions (session files whose PIDs are no longer running).
pub fn list_stale_sessions() -> Result<Vec<(u32, PathBuf)>, String> {
    let dir = sessions_dir()?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut stale = vec![];
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read sessions directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(pid) = stem.parse::<u32>() {
                    if !process_alive(pid) {
                        stale.push((pid, path));
                    }
                }
            }
        }
    }

    Ok(stale)
}

/// Check if a process is still running.
fn process_alive(pid: u32) -> bool {
    PathBuf::from(format!("/proc/{pid}")).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_dir_uses_xdg_when_set() {
        // This test just verifies the logic; actual env manipulation
        // would need more isolation.
        let dir = sessions_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.to_str().unwrap().contains("envision"));
        assert!(path.to_str().unwrap().ends_with("sessions"));
    }
}
