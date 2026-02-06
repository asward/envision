use crate::output::Output;
use crate::session::{self, OverwriteKind};
use crate::storage;

pub fn run(out: &Output, var: &str, value: &str) -> Result<(), String> {
    // 03-R2, 03-R3: validate POSIX variable name
    session::validate_var_name(var)?;

    // 03-R13: warn on system-critical variables
    if session::is_critical_var(var) {
        out.warn(&format!("Warning: '{var}' is a system-critical variable"));
    }

    // 03-R4, 03-R5, 03-R16: output shell export command to stdout
    let escaped = value.replace('\'', "'\\''");
    println!("export {var}='{escaped}'");

    // 03-R10: confirm the variable was set (to stderr)
    out.success(&format!("Set {var}={value}"));

    // 03-R6, 03-R7, 03-R8: track if session exists
    let pid = session::parent_pid();
    if storage::session_exists(pid)? {
        let mut sess = storage::load_session(pid)?;

        // 03-R14: skip tracking if value is identical to what's already tracked
        if let Some(session::TrackedChange::Set { value: tracked_val, .. }) = sess.tracked.get(var) {
            if tracked_val == value {
                return Ok(());
            }
        }

        let result = sess.track_set(var, value);
        storage::save_session(&sess)?;

        // 03-R11, 03-R12: display previous value and overwrite info
        if let Some(prev) = &result.previous {
            let kind = match &result.overwrite_kind {
                Some(OverwriteKind::Tracked) => " (was tracked)",
                Some(OverwriteKind::Untracked) => " (was untracked)",
                None => "",
            };
            out.key_value("Previous", &format!("{prev}{kind}"));
        }
    }

    Ok(())
}
