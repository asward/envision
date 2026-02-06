use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SESSION_VAR: &str = "ENVISION_SESSION";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: u64,
    /// Baseline: variable name -> hash of original value.
    pub baseline: BTreeMap<String, u64>,
    /// Tracked changes with full values.
    pub tracked: BTreeMap<String, TrackedChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Create a new session from the current environment.
    /// Stores only hashes of baseline values.
    pub fn new(env: &BTreeMap<String, String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_secs();

        let id = generate_session_id(std::process::id(), now);

        let baseline = env
            .iter()
            .filter(|(k, _)| k.as_str() != SESSION_VAR)
            .map(|(k, v)| (k.clone(), hash_value(v)))
            .collect();

        Self {
            id,
            created_at: now,
            baseline,
            tracked: BTreeMap::new(),
        }
    }

    /// Encode session as base64 string for storing in an env var.
    pub fn encode(&self) -> Result<String, String> {
        let json = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize session: {e}"))?;
        Ok(STANDARD.encode(json.as_bytes()))
    }

    /// Decode session from a base64 env var value.
    pub fn decode(encoded: &str) -> Result<Self, String> {
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|e| format!("Session data corrupted (bad base64): {e}"))?;
        let json = std::str::from_utf8(&bytes)
            .map_err(|e| format!("Session data corrupted (bad utf8): {e}"))?;
        serde_json::from_str(json)
            .map_err(|e| format!("Session data corrupted (bad json): {e}"))
    }

    /// Load session from the ENVISION_SESSION env var, if present.
    pub fn load() -> Result<Option<Self>, String> {
        match std::env::var(SESSION_VAR) {
            Ok(val) if !val.is_empty() => Ok(Some(Self::decode(&val)?)),
            _ => Ok(None),
        }
    }

    /// Return the shell export statement to persist this session.
    pub fn export_statement(&self) -> Result<String, String> {
        let encoded = self.encode()?;
        Ok(format!("export {SESSION_VAR}='{encoded}'"))
    }

    /// Record a set operation. Returns info about what was overwritten.
    /// 03-R6, 03-R7, 03-R8
    pub fn track_set(&mut self, var: &str, value: &str) -> SetResult {
        let previous = self.tracked_value(var);

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
        let previous = self.tracked_value(var);

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

    /// Get the last known value from tracked changes.
    /// Since baseline only stores hashes, we can only return values
    /// from tracked changes (which store full values).
    fn tracked_value(&self, var: &str) -> Option<String> {
        match self.tracked.get(var) {
            Some(TrackedChange::Set { value, .. }) => Some(value.clone()),
            Some(TrackedChange::Unset { .. }) => None,
            None => None,
        }
    }

    /// Check if a variable existed in the baseline (by name).
    pub fn in_baseline(&self, var: &str) -> bool {
        self.baseline.contains_key(var)
    }

    /// Check if a baseline variable's value has changed.
    /// Compares current env value hash against stored baseline hash.
    pub fn baseline_changed(&self, var: &str, current_value: &str) -> bool {
        match self.baseline.get(var) {
            Some(&baseline_hash) => hash_value(current_value) != baseline_hash,
            None => false,
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

/// Env vars managed by envision that should be excluded from untracked change detection.
const ENVISION_VARS: &[&str] = &[
    SESSION_VAR,
    "ENVISION_PROFILE",
    "ENVISION_PROFILE_CHECKSUM",
    "ENVISION_SESSION_ID",
    "ENVISION_TRACKED",
    "ENVISION_DIRTY",
];

/// Count environment changes not tracked by the session.
/// An untracked change is a baseline variable whose current hash differs
/// from the stored hash, AND which is not in the tracked set.
/// Also counts baseline variables that have disappeared and new variables
/// that weren't in the baseline (excluding envision-managed vars and tracked vars).
pub fn count_untracked(session: &Session, current_env: &BTreeMap<String, String>) -> usize {
    let mut count = 0;

    // Check baseline vars for hash changes or disappearance
    for (var, &baseline_hash) in &session.baseline {
        if session.tracked.contains_key(var) {
            continue;
        }
        match current_env.get(var) {
            Some(current_val) => {
                if hash_value(current_val) != baseline_hash {
                    count += 1;
                }
            }
            None => {
                count += 1;
            }
        }
    }

    // Check for new variables not in baseline and not tracked
    for var in current_env.keys() {
        if ENVISION_VARS.iter().any(|&v| v == var) {
            continue;
        }
        if session.baseline.contains_key(var) {
            continue;
        }
        if session.tracked.contains_key(var) {
            continue;
        }
        count += 1;
    }

    count
}

/// FNV-1a hash for value fingerprinting.
pub fn hash_value(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Validate that a variable name follows POSIX naming rules.
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

/// Check if a variable name is system-critical.
pub fn is_critical_var(name: &str) -> bool {
    CRITICAL_VARS.contains(&name)
}

fn generate_session_id(pid: u32, timestamp: u64) -> String {
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

    fn test_env() -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "bar".into());
        env.insert("PATH".into(), "/usr/bin".into());
        env
    }

    #[test]
    fn new_session_hashes_baseline() {
        let env = test_env();
        let session = Session::new(&env);
        assert_eq!(session.baseline.len(), 2);
        assert_eq!(*session.baseline.get("FOO").unwrap(), hash_value("bar"));
        assert!(session.tracked.is_empty());
    }

    #[test]
    fn encode_decode_roundtrip() {
        let env = test_env();
        let session = Session::new(&env);
        let encoded = session.encode().unwrap();
        let decoded = Session::decode(&encoded).unwrap();
        assert_eq!(session.id, decoded.id);
        assert_eq!(session.baseline, decoded.baseline);
    }

    #[test]
    fn baseline_excludes_session_var() {
        let mut env = test_env();
        env.insert(SESSION_VAR.into(), "should_be_excluded".into());
        let session = Session::new(&env);
        assert!(!session.baseline.contains_key(SESSION_VAR));
    }

    #[test]
    fn baseline_changed_detection() {
        let env = test_env();
        let session = Session::new(&env);
        assert!(!session.baseline_changed("FOO", "bar"));
        assert!(session.baseline_changed("FOO", "baz"));
        assert!(!session.baseline_changed("NONEXISTENT", "whatever"));
    }

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
        let session = &mut Session::new(&BTreeMap::new());
        let result = session.track_set("FOO", "bar");
        assert!(result.previous.is_none());
        assert!(result.overwrite_kind.is_none());
        assert!(matches!(
            session.tracked.get("FOO"),
            Some(TrackedChange::Set { value, previous: None }) if value == "bar"
        ));
    }

    #[test]
    fn track_set_overwrites_tracked() {
        let session = &mut Session::new(&BTreeMap::new());
        session.track_set("FOO", "first");
        let result = session.track_set("FOO", "second");
        assert_eq!(result.previous.as_deref(), Some("first"));
        assert!(matches!(result.overwrite_kind, Some(OverwriteKind::Tracked)));
    }

    #[test]
    fn track_unset_original_variable() {
        let env = test_env();
        let session = &mut Session::new(&env);
        // Simulate: var exists in baseline, we need to supply previous value
        // via a prior track_set or by passing it in from the real env
        // Since baseline only has hashes, track_unset won't know the value
        // unless it was tracked. For original vars, the caller passes the value.
        let result = session.track_unset("FOO");
        // previous is None because tracked_value returns None for untracked vars
        assert!(result.previous.is_none());
        assert!(matches!(result.previous_kind, PreviousKind::Original));
    }

    #[test]
    fn track_unset_tracked_variable() {
        let session = &mut Session::new(&BTreeMap::new());
        session.track_set("FOO", "bar");
        let result = session.track_unset("FOO");
        assert_eq!(result.previous.as_deref(), Some("bar"));
        assert!(matches!(result.previous_kind, PreviousKind::Tracked));
    }

    #[test]
    fn track_unset_nonexistent_variable() {
        let session = &mut Session::new(&BTreeMap::new());
        let result = session.track_unset("FOO");
        assert!(result.previous.is_none());
        assert!(!session.tracked.contains_key("FOO"));
    }

    #[test]
    fn count_untracked_clean_when_matching() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));
        baseline.insert("BAZ".into(), hash_value("qux"));

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "bar".into());
        env.insert("BAZ".into(), "qux".into());

        assert_eq!(count_untracked(&session, &env), 0);
    }

    #[test]
    fn count_untracked_detects_changed_value() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "changed".into());

        assert_eq!(count_untracked(&session, &env), 1);
    }

    #[test]
    fn count_untracked_detects_removed_var() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked: BTreeMap::new(),
        };

        let env = BTreeMap::new();

        assert_eq!(count_untracked(&session, &env), 1);
    }

    #[test]
    fn count_untracked_detects_new_var() {
        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline: BTreeMap::new(),
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert("NEW_VAR".into(), "hello".into());

        assert_eq!(count_untracked(&session, &env), 1);
    }

    #[test]
    fn count_untracked_ignores_envision_vars() {
        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline: BTreeMap::new(),
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert(SESSION_VAR.into(), "data".into());
        env.insert("ENVISION_PROFILE".into(), "dev".into());
        env.insert("ENVISION_DIRTY".into(), "0".into());
        env.insert("ENVISION_SESSION_ID".into(), "abc".into());
        env.insert("ENVISION_TRACKED".into(), "0".into());

        assert_eq!(count_untracked(&session, &env), 0);
    }

    #[test]
    fn count_untracked_skips_tracked_vars() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));

        let mut tracked = BTreeMap::new();
        tracked.insert("FOO".into(), TrackedChange::Set {
            value: "changed".into(),
            previous: Some("bar".into()),
        });

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked,
        };

        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "changed".into());

        assert_eq!(count_untracked(&session, &env), 0);
    }
}
