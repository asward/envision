use crate::output::Output;
use crate::session::Session;
use crate::storage;
use std::collections::BTreeMap;

pub fn run(out: &Output, force: bool, resume: bool) -> Result<(), String> {
    let pid = parent_pid();
    let exists = storage::session_exists(pid)?;

    // 01-R9: --resume continues existing session
    if resume {
        if !exists {
            return Err("No existing session to resume. Run 'envision session init' first.".into());
        }
        let session = storage::load_session(pid)?;
        out.success("Session resumed");
        print_session_info(out, &session);
        return Ok(());
    }

    // 01-R7: error if session already exists (without --force)
    if exists && !force {
        return Err(
            "Session already exists for this shell. Use --force to reinitialize or --resume to continue.".into()
        );
    }

    // 01-R8: --force warns and reinitializes
    if exists && force {
        out.warn("Reinitializing session (previous tracking history will be lost)");
        storage::remove_session(pid)?;
    }

    // 01-R12: detect and report stale sessions
    let stale = storage::list_stale_sessions()?;
    if !stale.is_empty() {
        out.warn(&format!(
            "Found {} stale session(s) from previous shells (cleaning up)",
            stale.len()
        ));
        for (stale_pid, _path) in &stale {
            let _ = storage::remove_session(*stale_pid);
        }
    }

    // 01-R1: capture all current environment variables as baseline
    let env: BTreeMap<String, String> = std::env::vars().collect();
    let var_count = env.len();

    // 01-R3: generate unique session identifier
    // 01-R4: create storage
    // 01-R5: initialize empty tracking state
    // 01-R6: record timestamp and shell PID
    let session = Session::new(pid, env);
    let storage_path = storage::save_session(&session)?;

    // 01-R10: display results
    out.success("Session initialized");
    print_session_info(out, &session);
    out.key_value("Variables captured", &var_count.to_string());
    out.key_value("Storage", &storage_path.display().to_string());

    Ok(())
}

fn print_session_info(out: &Output, session: &Session) {
    out.key_value("Session", &session.id);
    out.key_value("PID", &session.pid.to_string());
}

/// Get the parent shell's PID. The envision binary is invoked as a child
/// process, so its parent is the shell we want to track.
fn parent_pid() -> u32 {
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
