use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub pid: u32,
    pub created_at: u64,
    pub baseline: BTreeMap<String, String>,
    pub tracked: BTreeMap<String, TrackedChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TrackedChange {
    Set {
        value: String,
        previous: Option<String>,
    },
    Unset {
        previous: String,
    },
}

/// System-critical variables that warrant a warning before modification.
const CRITICAL_VARS: &[&str] = &[
    "PATH", "HOME", "USER", "SHELL", "TERM", "LANG", "PWD", "OLDPWD",
    "LD_LIBRARY_PATH", "LD_PRELOAD",
];

impl Session {
    pub fn new(pid: u32, env: BTreeMap<String, String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_secs();

        let id = generate_session_id(pid, now);

        Self {
            id,
            pid,
            created_at: now,
            baseline: env,
            tracked: BTreeMap::new(),
        }
    }

    /// Record a set operation. Returns info about what was overwritten.
    /// 03-R6, 03-R7, 03-R8
    pub fn track_set(&mut self, var: &str, value: &str) -> SetResult {
        let previous = self.current_value(var);

        let overwrite_kind = if self.tracked.contains_key(var) {
            Some(OverwriteKind::Tracked)
        } else if previous.is_some() {
            Some(OverwriteKind::Untracked)
        } else {
            None
        };

        self.tracked.insert(var.to_string(), TrackedChange::Set {
            value: value.to_string(),
            previous: previous.clone(),
        });

        SetResult { previous, overwrite_kind }
    }

    /// Record an unset operation. Returns info about what was removed.
    /// 04-R4, 04-R5, 04-R6
    pub fn track_unset(&mut self, var: &str) -> UnsetResult {
        let previous = self.current_value(var);

        let previous_kind = if self.tracked.contains_key(var) {
            PreviousKind::Tracked
        } else if self.baseline.contains_key(var) {
            PreviousKind::Original
        } else {
            PreviousKind::Untracked
        };

        if let Some(prev) = &previous {
            self.tracked.insert(var.to_string(), TrackedChange::Unset {
                previous: prev.clone(),
            });
        }

        UnsetResult { previous, previous_kind }
    }

    /// Get the current effective value of a variable:
    /// check tracked changes first, then baseline.
    fn current_value(&self, var: &str) -> Option<String> {
        match self.tracked.get(var) {
            Some(TrackedChange::Set { value, .. }) => Some(value.clone()),
            Some(TrackedChange::Unset { .. }) => None,
            None => self.baseline.get(var).cloned(),
        }
    }
}

pub struct SetResult {
    pub previous: Option<String>,
    pub overwrite_kind: Option<OverwriteKind>,
}

pub enum OverwriteKind {
    Tracked,
    Untracked,
}

pub struct UnsetResult {
    pub previous: Option<String>,
    pub previous_kind: PreviousKind,
}

pub enum PreviousKind {
    Tracked,
    Original,
    Untracked,
}

/// Validate that a variable name follows POSIX naming rules:
/// starts with letter or underscore, contains only letters, digits, underscores.
/// 03-R2
pub fn validate_var_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Variable name cannot be empty".into());
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(format!(
            "Invalid variable name '{name}': must start with a letter or underscore"
        ));
    }
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(format!(
                "Invalid variable name '{name}': contains invalid character '{c}'"
            ));
        }
    }
    Ok(())
}

/// Check if a variable name is system-critical. 03-R13
pub fn is_critical_var(name: &str) -> bool {
    CRITICAL_VARS.contains(&name)
}

/// Get the parent shell's PID.
pub fn parent_pid() -> u32 {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            safe fn getppid() -> u32;
        }
        getppid()
    }
    #[cfg(not(unix))]
    {
        std::process::id()
    }
}

fn generate_session_id(pid: u32, timestamp: u64) -> String {
    // Mix PID and timestamp bits, take 8 hex chars
    let mut h: u64 = 0x517cc1b727220a95;
    h ^= pid as u64;
    h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= timestamp;
    h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= h >> 27;
    format!("{:08x}", (h >> 32) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_8_hex_chars() {
        let id = generate_session_id(1234, 1700000000);
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_inputs_produce_different_ids() {
        let a = generate_session_id(1234, 1700000000);
        let b = generate_session_id(1235, 1700000000);
        let c = generate_session_id(1234, 1700000001);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn valid_posix_var_names() {
        assert!(validate_var_name("FOO").is_ok());
        assert!(validate_var_name("_BAR").is_ok());
        assert!(validate_var_name("a1_2").is_ok());
        assert!(validate_var_name("_").is_ok());
    }

    #[test]
    fn invalid_posix_var_names() {
        assert!(validate_var_name("").is_err());
        assert!(validate_var_name("1FOO").is_err());
        assert!(validate_var_name("FOO-BAR").is_err());
        assert!(validate_var_name("FOO.BAR").is_err());
        assert!(validate_var_name("FOO BAR").is_err());
    }

    #[test]
    fn critical_vars_detected() {
        assert!(is_critical_var("PATH"));
        assert!(is_critical_var("HOME"));
        assert!(!is_critical_var("MY_CUSTOM_VAR"));
    }

    #[test]
    fn track_set_new_variable() {
        let env = BTreeMap::new();
        let mut session = Session::new(1, env);
        let result = session.track_set("FOO", "bar");
        assert!(result.previous.is_none());
        assert!(result.overwrite_kind.is_none());
        assert!(matches!(
            session.tracked.get("FOO"),
            Some(TrackedChange::Set { value, previous: None }) if value == "bar"
        ));
    }

    #[test]
    fn track_set_overwrites_baseline() {
        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "old".into());
        let mut session = Session::new(1, env);
        let result = session.track_set("FOO", "new");
        assert_eq!(result.previous.as_deref(), Some("old"));
        assert!(matches!(result.overwrite_kind, Some(OverwriteKind::Untracked)));
    }

    #[test]
    fn track_unset_original_variable() {
        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "bar".into());
        let mut session = Session::new(1, env);
        let result = session.track_unset("FOO");
        assert_eq!(result.previous.as_deref(), Some("bar"));
        assert!(matches!(result.previous_kind, PreviousKind::Original));
        assert!(matches!(
            session.tracked.get("FOO"),
            Some(TrackedChange::Unset { previous }) if previous == "bar"
        ));
    }

    #[test]
    fn track_unset_tracked_variable() {
        let env = BTreeMap::new();
        let mut session = Session::new(1, env);
        session.track_set("FOO", "bar");
        let result = session.track_unset("FOO");
        assert_eq!(result.previous.as_deref(), Some("bar"));
        assert!(matches!(result.previous_kind, PreviousKind::Tracked));
    }

    #[test]
    fn track_unset_nonexistent_variable() {
        let env = BTreeMap::new();
        let mut session = Session::new(1, env);
        let result = session.track_unset("FOO");
        assert!(result.previous.is_none());
        // Should not insert a tracking entry
        assert!(!session.tracked.contains_key("FOO"));
    }

    #[test]
    fn track_set_overwrites_tracked() {
        let env = BTreeMap::new();
        let mut session = Session::new(1, env);
        session.track_set("FOO", "first");
        let result = session.track_set("FOO", "second");
        assert_eq!(result.previous.as_deref(), Some("first"));
        assert!(matches!(result.overwrite_kind, Some(OverwriteKind::Tracked)));
    }
}
