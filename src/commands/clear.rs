use crate::export::Exports;
use crate::output::Output;
use crate::session::{Session, TrackedChange};
use std::io::{self, IsTerminal, Write};

/// 05-R1 through 05-R14
pub fn run(out: &Output, ex: &mut Exports, force: bool) -> Result<u8, String> {
    // 05-R1, 05-R13: require active session with baseline
    let mut session = Session::load()?
        .ok_or("No active session. Run 'envision session init' first.")?;

    // 05-R12: nothing to clear
    if session.tracked.is_empty() {
        out.success("Nothing to clear");
        return Ok(0);
    }

    // 05-R3: preview changes before applying
    let (to_unset, to_restore) = preview_changes(&session);

    out.info(&format!("{} tracked change(s) to clear:", session.tracked.len()));
    for var in &to_unset {
        out.info(&format!("  unset {var}"));
    }
    for (var, value) in &to_restore {
        out.info(&format!("  restore {var}={value}"));
    }

    // 05-R2: require confirmation unless --force
    if !force {
        prompt_confirmation()?;
    }

    // 05-R5: remove variables that were set through the tool
    for var in &to_unset {
        ex.unset_var(var);
    }

    // 05-R6: restore variables that were unset through the tool
    for (var, value) in &to_restore {
        ex.set_var(var, value);
    }

    // Clear tracked state in session
    session.tracked.clear();
    ex.save_session(&session)?;

    // 05-R9, 05-R10, 05-R11: display results
    if !to_unset.is_empty() {
        out.key_value("Removed", &to_unset.len().to_string());
    }
    if !to_restore.is_empty() {
        out.key_value("Restored", &to_restore.len().to_string());
    }
    out.success("State: clean");

    Ok(0)
}

/// Separate tracked changes into variables to unset and variables to restore.
/// 05-R5: Set vars get unset (they were added by the tool).
/// 05-R6: Unset vars get restored to their previous value.
fn preview_changes(session: &Session) -> (Vec<String>, Vec<(String, String)>) {
    let mut to_unset = Vec::new();
    let mut to_restore = Vec::new();

    for (var, change) in &session.tracked {
        match change {
            TrackedChange::Set { previous, .. } => {
                match previous {
                    // Was set over an existing value — restore the previous
                    Some(prev) => to_restore.push((var.clone(), prev.clone())),
                    // Was newly added — remove it
                    None => to_unset.push(var.clone()),
                }
            }
            TrackedChange::Unset { previous } => {
                // Was unset — restore it
                to_restore.push((var.clone(), previous.clone()));
            }
        }
    }

    to_unset.sort();
    to_restore.sort();
    (to_unset, to_restore)
}

/// 05-R2: interactive confirmation prompt.
fn prompt_confirmation() -> Result<(), String> {
    if !io::stdin().is_terminal() {
        return Err("Cannot prompt for confirmation: not a terminal. Use --force to skip.".into());
    }

    eprint!("Clear all tracked changes? [y/N] ");
    io::stderr().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {e}"))?;

    if input.trim().eq_ignore_ascii_case("y") {
        Ok(())
    } else {
        Err("Clear cancelled".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn session_with_tracked() -> Session {
        let mut baseline = BTreeMap::new();
        baseline.insert("EXISTING".into(), crate::session::hash_value("original"));

        let mut tracked = BTreeMap::new();
        // A newly added var (no previous)
        tracked.insert("NEW_VAR".into(), TrackedChange::Set {
            value: "added".into(),
            previous: None,
        });
        // An overwritten var (has previous)
        tracked.insert("EXISTING".into(), TrackedChange::Set {
            value: "changed".into(),
            previous: Some("original".into()),
        });
        // An unset var
        tracked.insert("REMOVED".into(), TrackedChange::Unset {
            previous: "was_here".into(),
        });

        Session {
            id: "test1234".into(),
            created_at: 0,
            baseline,
            tracked,
        }
    }

    #[test]
    fn preview_new_var_gets_unset() {
        let session = session_with_tracked();
        let (to_unset, _) = preview_changes(&session);
        assert!(to_unset.contains(&"NEW_VAR".to_string()));
    }

    #[test]
    fn preview_overwritten_var_gets_restored() {
        let session = session_with_tracked();
        let (_, to_restore) = preview_changes(&session);
        assert!(to_restore.iter().any(|(k, v)| k == "EXISTING" && v == "original"));
    }

    #[test]
    fn preview_unset_var_gets_restored() {
        let session = session_with_tracked();
        let (_, to_restore) = preview_changes(&session);
        assert!(to_restore.iter().any(|(k, v)| k == "REMOVED" && v == "was_here"));
    }

    #[test]
    fn preview_counts() {
        let session = session_with_tracked();
        let (to_unset, to_restore) = preview_changes(&session);
        assert_eq!(to_unset.len(), 1);
        assert_eq!(to_restore.len(), 2);
    }

    #[test]
    fn preview_empty_session() {
        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline: BTreeMap::new(),
            tracked: BTreeMap::new(),
        };
        let (to_unset, to_restore) = preview_changes(&session);
        assert!(to_unset.is_empty());
        assert!(to_restore.is_empty());
    }
}
