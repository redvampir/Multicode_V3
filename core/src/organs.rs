use std::env;
use std::time::Duration;

/// Reads stage delays for the organs builder from environment variables.
///
/// Supports two variable names:
/// - `ORGANS_BUILDER_STAGE_DELAYS`
/// - `ORGANS_BUILDER_STAGE_DELAYS_MS` (interpreted as milliseconds, kept for backward compatibility)
///
/// The value should be a comma-separated list of integers representing delays in milliseconds.
/// Returns `None` if neither variable is set or if parsing fails.
pub fn stage_delays_from_env() -> Option<Vec<Duration>> {
    if let Ok(value) = env::var("ORGANS_BUILDER_STAGE_DELAYS") {
        parse(&value)
    } else if let Ok(value) = env::var("ORGANS_BUILDER_STAGE_DELAYS_MS") {
        parse(&value)
    } else {
        None
    }
}

fn parse(value: &str) -> Option<Vec<Duration>> {
    let mut out = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let ms: u64 = part.parse().ok()?;
        out.push(Duration::from_millis(ms));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_new_variable() {
        env::remove_var("ORGANS_BUILDER_STAGE_DELAYS_MS");
        env::set_var("ORGANS_BUILDER_STAGE_DELAYS", "10,20");
        let delays = stage_delays_from_env().unwrap();
        env::remove_var("ORGANS_BUILDER_STAGE_DELAYS");
        assert_eq!(
            delays,
            vec![Duration::from_millis(10), Duration::from_millis(20)]
        );
    }

    #[test]
    fn reads_legacy_variable() {
        env::remove_var("ORGANS_BUILDER_STAGE_DELAYS");
        env::set_var("ORGANS_BUILDER_STAGE_DELAYS_MS", "5,15");
        let delays = stage_delays_from_env().unwrap();
        env::remove_var("ORGANS_BUILDER_STAGE_DELAYS_MS");
        assert_eq!(
            delays,
            vec![Duration::from_millis(5), Duration::from_millis(15)]
        );
    }
}
