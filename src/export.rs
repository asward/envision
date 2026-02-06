use crate::session::{self, Session};
use std::collections::BTreeMap;

pub const SESSION_ID_VAR: &str = "ENVISION_SESSION_ID";
pub const TRACKED_COUNT_VAR: &str = "ENVISION_TRACKED";
pub const DIRTY_VAR: &str = "ENVISION_DIRTY";

/// Collects shell statements to be eval'd by the hook.
/// All stdout output goes through here.
pub struct Exports {
    statements: Vec<String>,
    /// Most recently saved session, used by update_banner_vars() to avoid
    /// reading the stale ENVISION_SESSION env var from the parent shell.
    last_session: Option<Session>,
}

impl Exports {
    pub fn new() -> Self {
        Self { statements: Vec::new(), last_session: None }
    }

    /// Queue `export VAR='value'`, escaping single quotes in the value.
    pub fn set_var(&mut self, var: &str, value: &str) {
        let escaped = value.replace('\'', "'\\''");
        self.statements.push(format!("export {var}='{escaped}'"));
    }

    /// Queue `unset VAR`.
    pub fn unset_var(&mut self, var: &str) {
        self.statements.push(format!("unset {var}"));
    }

    /// Queue the session env var export.
    pub fn save_session(&mut self, session: &Session) -> Result<(), String> {
        self.statements.push(session.export_statement()?);
        self.last_session = Some(session.clone());
        Ok(())
    }

    /// Compute and queue banner state env vars (session ID, tracked count, dirty flag).
    /// Uses the last saved session if available, otherwise loads from env.
    pub fn update_banner_vars(&mut self) -> Result<(), String> {
        let session = match self.last_session.take() {
            Some(s) => Some(s),
            None => Session::load()?,
        };

        match session {
            Some(session) => {
                self.set_var(SESSION_ID_VAR, &session.id);
                self.set_var(TRACKED_COUNT_VAR, &session.tracked.len().to_string());

                let current_env: BTreeMap<String, String> = std::env::vars().collect();
                let untracked = session::count_untracked(&session, &current_env);
                let dirty = if untracked > 0 { "1" } else { "0" };
                self.set_var(DIRTY_VAR, dirty);
            }
            None => {
                self.unset_var(SESSION_ID_VAR);
                self.unset_var(TRACKED_COUNT_VAR);
                self.unset_var(DIRTY_VAR);
            }
        }
        Ok(())
    }

    /// Write all queued statements to stdout.
    pub fn flush(self) {
        for stmt in &self.statements {
            println!("{stmt}");
        }
    }
}
