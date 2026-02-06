use crate::export::Exports;
use crate::output::Output;
use crate::session::Session;
use std::collections::BTreeMap;

pub fn init(out: &Output, ex: &mut Exports, force: bool, resume: bool) -> Result<u8, String> {
    let existing = Session::load()?;

    // 01-R9: --resume continues existing session
    if resume {
        match existing {
            Some(session) => {
                out.success("Session resumed");
                out.key_value("Session", &session.id);
                return Ok(0);
            }
            None => {
                return Err("No existing session to resume. Run 'envision session init' first.".into());
            }
        }
    }

    // 01-R7: error if session already exists (without --force)
    if existing.is_some() && !force {
        return Err(
            "Session already exists. Use --force to reinitialize or --resume to continue.".into()
        );
    }

    // 01-R8: --force warns and reinitializes
    if existing.is_some() && force {
        out.warn("Reinitializing session (previous tracking history will be lost)");
    }

    // 01-R1: capture all current environment variables as baseline (hashed)
    let env: BTreeMap<String, String> = std::env::vars().collect();

    // 01-R3: generate unique session identifier
    // 01-R5: initialize empty tracking state
    // 01-R6: record timestamp
    let session = Session::new(&env);
    ex.save_session(&session)?;

    // 01-R10: display results (to stderr)
    // 01-R13: banner is activated via update_banner_vars() in main.rs
    out.success("Session initialized");
    out.key_value("Session", &session.id);

    Ok(0)
}

/// Ensure a session exists, creating one if needed. Returns the active session.
/// Used by profile (08-R1) and any command that requires an active session.
pub fn ensure_session(out: &Output, ex: &mut Exports) -> Result<Session, String> {
    if let Some(session) = Session::load()? {
        return Ok(session);
    }

    let env: BTreeMap<String, String> = std::env::vars().collect();
    let session = Session::new(&env);
    ex.save_session(&session)?;

    out.success("Session initialized");
    out.key_value("Session", &session.id);

    Ok(session)
}
