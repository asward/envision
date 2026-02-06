use crate::output::Output;
use crate::session::{self, Session};
use std::collections::BTreeMap;

/// 02-R1 through 02-R11
pub fn run(out: &Output) -> Result<u8, String> {
    // 02-R1: error if no session
    // 02-R9: flag if baseline missing/corrupted (handled by Session::load error path)
    let session = Session::load()?
        .ok_or("No active session. Run 'envision session init' first.")?;

    // 02-R3: display baseline timestamp
    let timestamp = format_timestamp(session.created_at);
    out.key_value("Session", &session.id);
    out.key_value("Baseline", &timestamp);

    // 02-R4: count tracked variables
    let tracked_count = session.tracked.len();
    out.key_value("Tracked changes", &tracked_count.to_string());

    // 02-R5: count untracked changes
    let current_env: BTreeMap<String, String> = std::env::vars().collect();
    let untracked_count = session::count_untracked(&session, &current_env);
    out.key_value("Untracked changes", &untracked_count.to_string());

    // 02-R6: total count differing from baseline
    let total_changed = tracked_count + untracked_count;
    out.key_value("Total changed", &total_changed.to_string());

    // 02-R7, 02-R8: dirty/clean state
    let dirty = untracked_count > 0;
    if dirty {
        out.warn("State: dirty");
    } else {
        out.success("State: clean");
    }

    // 02-R10, 02-R11: exit code
    if dirty { Ok(1) } else { Ok(0) }
}

fn format_timestamp(epoch_secs: u64) -> String {
    let secs = epoch_secs;
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Civil date from day count (algorithm from Howard Hinnant)
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02} UTC")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_known_date() {
        assert_eq!(format_timestamp(1704067200), "2024-01-01 00:00:00 UTC");
    }

    #[test]
    fn format_timestamp_with_time() {
        assert_eq!(format_timestamp(1707142995), "2024-02-05 14:23:15 UTC");
    }
}
