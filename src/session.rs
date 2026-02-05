use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub pid: u32,
    pub created_at: u64,
    pub baseline: BTreeMap<String, String>,
    pub tracked: BTreeMap<String, TrackedChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TrackedChange {
    Set {
        value: String,
        previous: Option<String>,
    },
    Unset {
        previous: String,
    },
}

impl Session {
    pub fn new(pid: u32, env: BTreeMap<String, String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_secs();

        let id = generate_session_id(pid, now);

        Self {
            id,
            pid,
            created_at: now,
            baseline: env,
            tracked: BTreeMap::new(),
        }
    }
}

fn generate_session_id(pid: u32, timestamp: u64) -> String {
    // Mix PID and timestamp bits, take 8 hex chars
    let mut h: u64 = 0x517cc1b727220a95;
    h ^= pid as u64;
    h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= timestamp;
    h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= h >> 27;
    format!("{:08x}", (h >> 32) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_8_hex_chars() {
        let id = generate_session_id(1234, 1700000000);
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_inputs_produce_different_ids() {
        let a = generate_session_id(1234, 1700000000);
        let b = generate_session_id(1235, 1700000000);
        let c = generate_session_id(1234, 1700000001);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
