use crate::output::Output;
use crate::session::{Session, hash_value, SESSION_VAR};
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
    // Compare current env against baseline, excluding tracked vars and SESSION_VAR
    let current_env: BTreeMap<String, String> = std::env::vars().collect();
    let untracked_count = count_untracked(&session, &current_env);
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

/// Count environment changes not tracked by the session.
/// An untracked change is a baseline variable whose current hash differs
/// from the stored hash, AND which is not in the tracked set.
/// Also counts baseline variables that have disappeared and new variables
/// that weren't in the baseline (excluding SESSION_VAR and tracked vars).
fn count_untracked(session: &Session, current_env: &BTreeMap<String, String>) -> usize {
    let mut count = 0;

    // Check baseline vars for hash changes or disappearance
    for (var, &baseline_hash) in &session.baseline {
        if session.tracked.contains_key(var) {
            continue;
        }
        match current_env.get(var) {
            Some(current_val) => {
                if hash_value(current_val) != baseline_hash {
                    count += 1;
                }
            }
            None => {
                // Variable was in baseline but is now gone (and not tracked)
                count += 1;
            }
        }
    }

    // Check for new variables not in baseline and not tracked
    for var in current_env.keys() {
        if var == SESSION_VAR {
            continue;
        }
        if session.baseline.contains_key(var) {
            continue;
        }
        if session.tracked.contains_key(var) {
            continue;
        }
        // New variable appeared that wasn't in baseline
        count += 1;
    }

    count
}

fn format_timestamp(epoch_secs: u64) -> String {
    // Convert epoch seconds to a human-readable UTC timestamp
    // Without pulling in chrono, do basic formatting
    let secs = epoch_secs;
    // Days from epoch to groups of 400/100/4/1 years
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Civil date from day count (algorithm from Howard Hinnant)
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
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
    use crate::session::TrackedChange;

    #[test]
    fn format_timestamp_known_date() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        assert_eq!(format_timestamp(1704067200), "2024-01-01 00:00:00 UTC");
    }

    #[test]
    fn format_timestamp_with_time() {
        // 2024-02-05 14:23:15 UTC = 1707142995
        assert_eq!(format_timestamp(1707142995), "2024-02-05 14:23:15 UTC");
    }

    #[test]
    fn untracked_clean_when_matching() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));
        baseline.insert("BAZ".into(), hash_value("qux"));

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "bar".into());
        env.insert("BAZ".into(), "qux".into());

        assert_eq!(count_untracked(&session, &env), 0);
    }

    #[test]
    fn untracked_detects_changed_value() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "changed".into());

        assert_eq!(count_untracked(&session, &env), 1);
    }

    #[test]
    fn untracked_detects_removed_var() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked: BTreeMap::new(),
        };

        let env = BTreeMap::new(); // FOO is gone

        assert_eq!(count_untracked(&session, &env), 1);
    }

    #[test]
    fn untracked_detects_new_var() {
        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline: BTreeMap::new(),
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert("NEW_VAR".into(), "hello".into());

        assert_eq!(count_untracked(&session, &env), 1);
    }

    #[test]
    fn untracked_ignores_session_var() {
        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline: BTreeMap::new(),
            tracked: BTreeMap::new(),
        };

        let mut env = BTreeMap::new();
        env.insert(SESSION_VAR.into(), "some_data".into());

        assert_eq!(count_untracked(&session, &env), 0);
    }

    #[test]
    fn untracked_skips_tracked_vars() {
        let mut baseline = BTreeMap::new();
        baseline.insert("FOO".into(), hash_value("bar"));

        let mut tracked = BTreeMap::new();
        tracked.insert("FOO".into(), TrackedChange::Set {
            value: "changed".into(),
            previous: Some("bar".into()),
        });

        let session = Session {
            id: "test".into(),
            created_at: 0,
            baseline,
            tracked,
        };

        let mut env = BTreeMap::new();
        env.insert("FOO".into(), "changed".into());

        // FOO changed but it's tracked, so untracked count should be 0
        assert_eq!(count_untracked(&session, &env), 0);
    }
}
