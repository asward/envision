use crate::session::Session;

/// Print the banner line to stdout for testing/debugging.
/// In normal use, the shell hook renders the banner directly from env vars.
pub fn run() -> Result<u8, String> {
    let profile = std::env::var("ENVISION_PROFILE").unwrap_or_default();
    let session = Session::load().ok().flatten();

    if profile.is_empty() && session.is_none() {
        return Ok(1);
    }

    let columns: usize = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);

    let no_color = std::env::var("NO_COLOR").is_ok()
        || std::env::args().any(|a| a == "--no-color");

    let content = render_content(&profile, session.as_ref());
    let visible_len = content.len();
    let pad = columns.saturating_sub(visible_len);
    let padded = format!("{content}{}", " ".repeat(pad));

    if no_color {
        println!("{padded}");
    } else {
        println!("\x1b[44;1;37m{padded}\x1b[0m");
    }

    Ok(0)
}

fn render_content(profile: &str, session: Option<&Session>) -> String {
    let mut parts = Vec::new();

    if !profile.is_empty() {
        parts.push(format!(" {profile}"));
    }

    if let Some(sess) = session {
        let tracked = sess.tracked.len();
        let dirty = std::env::var("ENVISION_DIRTY").unwrap_or_else(|_| "0".into());
        let state = if dirty == "1" { "dirty" } else { "clean" };
        parts.push(format!("{}  {}  {}", sess.id, tracked, state));
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("{} ", parts.join(" | "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_profile_only() {
        let content = render_content("dev", None);
        assert!(content.contains("dev"));
    }

    #[test]
    fn render_empty_when_nothing_active() {
        let content = render_content("", None);
        assert!(content.is_empty());
    }

    #[test]
    fn render_with_session() {
        use std::collections::BTreeMap;
        let session = Session {
            id: "abc123".into(),
            created_at: 0,
            baseline: BTreeMap::new(),
            tracked: BTreeMap::new(),
        };
        let content = render_content("dev", Some(&session));
        assert!(content.contains("dev"));
        assert!(content.contains("abc123"));
        assert!(content.contains("clean"));
    }
}
