use crate::output::Output;
use crate::session::{self, PreviousKind};
use crate::storage;

pub fn run(out: &Output, var: &str) -> Result<(), String> {
    // 04-R2: validate variable name exists in environment
    let current_value = std::env::var(var).ok();

    // 04-R12: warn but succeed if variable doesn't exist
    if current_value.is_none() {
        out.warn(&format!("Variable '{var}' is not set"));
        return Ok(());
    }

    // 04-R11: strong warning for system-critical variables
    if session::is_critical_var(var) {
        out.warn(&format!("Warning: '{var}' is a system-critical variable"));
    }

    // 04-R3: output shell unset command to stdout
    println!("unset {var}");

    // 04-R8, 04-R9: confirm and display removed value
    let prev = current_value.unwrap();
    out.success(&format!("Unset {var} (was: {prev})"));

    // 04-R4, 04-R5, 04-R6: track if session exists
    let pid = session::parent_pid();
    if storage::session_exists(pid)? {
        let mut sess = storage::load_session(pid)?;

        let result = sess.track_unset(var);
        storage::save_session(&sess)?;

        // 04-R10: indicate whether it was tracked, untracked, or original
        if result.previous.is_some() {
            let kind = match result.previous_kind {
                PreviousKind::Tracked => "tracked",
                PreviousKind::Original => "original",
                PreviousKind::Untracked => "untracked",
            };
            out.key_value("Was", kind);
        }
    }

    Ok(())
}
