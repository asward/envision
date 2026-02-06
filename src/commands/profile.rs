use crate::export::Exports;
use crate::output::Output;
use crate::session::{hash_value, Session, SESSION_VAR};
use std::collections::BTreeMap;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

const PROFILE_VAR: &str = "ENVISION_PROFILE";
const CHECKSUM_VAR: &str = "ENVISION_PROFILE_CHECKSUM";

/// Variables that inherently differ in a bash subshell — not real changes.
const SUBSHELL_NOISE: &[&str] = &["_", "SHLVL", "BASH_EXECUTION_STRING"];

/// 08-R2 through 08-R33
pub fn run(out: &Output, ex: &mut Exports, path: &str, yes: bool, dry_run: bool) -> Result<u8, String> {
    // 08-R31, 08-R32: resolve path
    let path = resolve_path(path);

    // 08-R2, 08-R3: verify file exists
    if !path.exists() {
        return Err(format!("Profile file not found: {}", path.display()));
    }

    // 08-R4, 08-R5: validate extension
    validate_extension(&path)?;

    // 08-R6, 08-R7: confirmation prompt on first load (no checksum stored)
    if !yes && std::env::var(CHECKSUM_VAR).is_err() {
        prompt_confirmation(out, &path)?;
    }

    // Capture current env
    let before: BTreeMap<String, String> = std::env::vars().collect();

    // Execute profile script in subshell, capture resulting env
    let after = execute_profile(&path)?;

    // Compute diff, filtering noise
    let changes = compute_diff(&before, &after);

    // 08-R22, 08-R23: dry-run mode
    if dry_run {
        let name = resolve_profile_name(&path);
        out.info(&format!("Dry run for profile '{name}':"));
        if changes.is_empty() {
            out.info("  (no changes)");
        }
        for change in &changes {
            match change {
                EnvChange::Set(var, value) => out.info(&format!("  set {var}={value}")),
                EnvChange::Unset(var) => out.info(&format!("  unset {var}")),
            }
        }
        return Ok(0);
    }

    // Apply changes via Exports
    for change in &changes {
        match change {
            EnvChange::Set(var, value) => ex.set_var(var, value),
            EnvChange::Unset(var) => ex.unset_var(var),
        }
    }

    // 08-R8, 08-R11: set ENVISION_PROFILE
    let profile_name = resolve_profile_name(&path);
    ex.set_var(PROFILE_VAR, &profile_name);

    // 08-R24: compute and store file checksum
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile for checksum: {e}"))?;
    let checksum = hash_value(&contents);
    ex.set_var(CHECKSUM_VAR, &checksum.to_string());

    // 08-R20: if session exists, track all changes
    if let Some(mut sess) = Session::load()? {
        for change in &changes {
            match change {
                EnvChange::Set(var, value) => { sess.track_set(var, value); }
                EnvChange::Unset(var) => { sess.track_unset(var); }
            }
        }
        ex.save_session(&sess)?;
    }

    // 08-R21: display confirmation
    out.success(&format!("Profile '{profile_name}' loaded"));
    out.key_value("Variables changed", &changes.len().to_string());

    Ok(0)
}

enum EnvChange {
    Set(String, String),
    Unset(String),
}

/// 08-R31, 08-R32: resolve relative paths against CWD.
fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    }
}

/// 08-R4, 08-R5: validate file extension.
fn validate_extension(path: &Path) -> Result<(), String> {
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid file path: {}", path.display()))?;

    if name.ends_with(".profile.sh") || name.ends_with(".envision") {
        Ok(())
    } else {
        Err(format!(
            "Invalid profile extension: '{}'. Must be .profile.sh or .envision",
            path.display()
        ))
    }
}

/// 08-R6: prompt for confirmation on first load.
fn prompt_confirmation(out: &Output, path: &Path) -> Result<(), String> {
    if !io::stdin().is_terminal() {
        return Err("Cannot prompt for confirmation: not a terminal. Use --yes to skip.".into());
    }

    out.warn(&format!("Loading profile: {}", path.display()));
    eprint!("Continue? [y/N] ");
    io::stderr().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {e}"))?;

    if input.trim().eq_ignore_ascii_case("y") {
        Ok(())
    } else {
        Err("Profile loading cancelled".into())
    }
}

/// Execute the profile script in a bash subshell, return the resulting environment.
/// 08-R19: propagate script errors.
fn execute_profile(path: &Path) -> Result<BTreeMap<String, String>, String> {
    // Run: source the script (redirect its stdout to stderr so only env -0 hits stdout),
    // then dump the full env null-terminated.
    // --norc --noprofile avoids loading shell configs that would pollute the diff.
    let output = std::process::Command::new("bash")
        .arg("--norc")
        .arg("--noprofile")
        .arg("-c")
        .arg(r#". "$1" 1>&2 && env -0"#)
        .arg("_") // $0 placeholder
        .arg(path.as_os_str())
        .output()
        .map_err(|e| format!("Failed to execute profile: {e}"))?;

    // 08-R19: return script error message and exit code
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(1);
        return Err(format!(
            "Profile script failed (exit {}): {}",
            code,
            stderr.trim()
        ));
    }

    // Parse null-terminated env output
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in profile environment: {e}"))?;

    let mut env = BTreeMap::new();
    for entry in stdout.split('\0') {
        if entry.is_empty() {
            continue;
        }
        if let Some((key, value)) = entry.split_once('=') {
            env.insert(key.to_string(), value.to_string());
        }
    }

    Ok(env)
}

/// Compute the diff between before and after environments.
fn compute_diff(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> Vec<EnvChange> {
    let mut changes = Vec::new();

    // Variables that were added or changed
    for (key, new_val) in after {
        if should_skip(key) {
            continue;
        }
        match before.get(key) {
            Some(old_val) if old_val == new_val => {} // unchanged
            _ => changes.push(EnvChange::Set(key.clone(), new_val.clone())),
        }
    }

    // Variables that were removed
    for key in before.keys() {
        if should_skip(key) {
            continue;
        }
        if !after.contains_key(key) {
            changes.push(EnvChange::Unset(key.clone()));
        }
    }

    changes
}

/// Skip variables that are subshell noise or managed by envision.
fn should_skip(var: &str) -> bool {
    SUBSHELL_NOISE.contains(&var)
        || var == SESSION_VAR
        || var == PROFILE_VAR
        || var == CHECKSUM_VAR
}

/// 08-R11: derive profile name from filename (strip extension).
fn resolve_profile_name(path: &Path) -> String {
    // 08-R8: use existing ENVISION_PROFILE if set
    if let Ok(existing) = std::env::var(PROFILE_VAR) {
        if !existing.is_empty() {
            return existing;
        }
    }

    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Handle double extension: foo.profile.sh -> foo
    if let Some(stem) = name.strip_suffix(".profile.sh") {
        stem.to_string()
    } else if let Some(stem) = name.strip_suffix(".envision") {
        stem.to_string()
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_extensions() {
        assert!(validate_extension(Path::new("dev.profile.sh")).is_ok());
        assert!(validate_extension(Path::new("prod.envision")).is_ok());
        assert!(validate_extension(Path::new("/home/user/my.profile.sh")).is_ok());
    }

    #[test]
    fn invalid_extensions() {
        assert!(validate_extension(Path::new("dev.sh")).is_err());
        assert!(validate_extension(Path::new("dev.env")).is_err());
        assert!(validate_extension(Path::new("profile")).is_err());
        assert!(validate_extension(Path::new("dev.txt")).is_err());
    }

    #[test]
    fn profile_name_from_profile_sh() {
        let path = Path::new("/home/user/dev.profile.sh");
        // SAFETY: test-only, no concurrent threads
        unsafe { std::env::remove_var(PROFILE_VAR); }
        assert_eq!(resolve_profile_name(path), "dev");
    }

    #[test]
    fn profile_name_from_envision() {
        let path = Path::new("production.envision");
        // SAFETY: test-only, no concurrent threads
        unsafe { std::env::remove_var(PROFILE_VAR); }
        assert_eq!(resolve_profile_name(path), "production");
    }

    #[test]
    fn resolve_relative_path() {
        let cwd = std::env::current_dir().unwrap();
        let resolved = resolve_path("dev.profile.sh");
        assert_eq!(resolved, cwd.join("dev.profile.sh"));
    }

    #[test]
    fn resolve_absolute_path() {
        let resolved = resolve_path("/etc/dev.profile.sh");
        assert_eq!(resolved, PathBuf::from("/etc/dev.profile.sh"));
    }

    #[test]
    fn diff_detects_added_var() {
        let before = BTreeMap::new();
        let mut after = BTreeMap::new();
        after.insert("NEW".into(), "value".into());

        let changes = compute_diff(&before, &after);
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], EnvChange::Set(k, v) if k == "NEW" && v == "value"));
    }

    #[test]
    fn diff_detects_changed_var() {
        let mut before = BTreeMap::new();
        before.insert("FOO".into(), "old".into());
        let mut after = BTreeMap::new();
        after.insert("FOO".into(), "new".into());

        let changes = compute_diff(&before, &after);
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], EnvChange::Set(k, v) if k == "FOO" && v == "new"));
    }

    #[test]
    fn diff_detects_removed_var() {
        let mut before = BTreeMap::new();
        before.insert("GONE".into(), "value".into());
        let after = BTreeMap::new();

        let changes = compute_diff(&before, &after);
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], EnvChange::Unset(k) if k == "GONE"));
    }

    #[test]
    fn diff_ignores_unchanged() {
        let mut before = BTreeMap::new();
        before.insert("SAME".into(), "value".into());
        let mut after = BTreeMap::new();
        after.insert("SAME".into(), "value".into());

        let changes = compute_diff(&before, &after);
        assert!(changes.is_empty());
    }

    #[test]
    fn diff_skips_noise_vars() {
        let before = BTreeMap::new();
        let mut after = BTreeMap::new();
        after.insert("_".into(), "/usr/bin/env".into());
        after.insert("SHLVL".into(), "2".into());
        after.insert("BASH_EXECUTION_STRING".into(), "source ...".into());
        after.insert(SESSION_VAR.into(), "data".into());

        let changes = compute_diff(&before, &after);
        assert!(changes.is_empty());
    }
}
