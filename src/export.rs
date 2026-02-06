use crate::session::Session;

/// Collects shell statements to be eval'd by the hook.
/// All stdout output goes through here.
pub struct Exports {
    statements: Vec<String>,
}

impl Exports {
    pub fn new() -> Self {
        Self { statements: Vec::new() }
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
        Ok(())
    }

    /// Write all queued statements to stdout.
    pub fn flush(self) {
        for stmt in &self.statements {
            println!("{stmt}");
        }
    }
}
