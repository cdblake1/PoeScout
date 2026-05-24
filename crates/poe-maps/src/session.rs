//! Pure decision logic for the automatic session lifecycle, extracted from the
//! async poll loop so it can be unit-tested. The caller owns the wall-clock idle
//! timer and performs the stash snapshot + start/end on the returned action.

use crate::state::TrackerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Start,
    End,
    None,
}

/// Decide the session action from the current tracker state:
/// - In a map with no active session → `Start`.
/// - In a map (session already open) → `None`.
/// - Idle/Stopped with an open session, idle for at least the timeout → `End`.
/// - Otherwise `None`.
pub fn next_session_action(
    state: &TrackerState,
    has_active_session: bool,
    idle_elapsed_secs: u64,
    idle_timeout_secs: u64,
) -> SessionAction {
    match state {
        TrackerState::InMap { .. } => {
            if has_active_session {
                SessionAction::None
            } else {
                SessionAction::Start
            }
        }
        TrackerState::Idle { .. } | TrackerState::Stopped => {
            if has_active_session && idle_elapsed_secs >= idle_timeout_secs {
                SessionAction::End
            } else {
                SessionAction::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_map() -> TrackerState {
        TrackerState::InMap {
            map_name: "Strand".into(),
            area_level: Some(83),
            map_tier: Some(16),
            started_at: "2025-05-20T14:00:00".into(),
            deaths: 0,
        }
    }

    fn idle() -> TrackerState {
        TrackerState::Idle {
            since: "2025-05-20T14:00:00".into(),
            zone_name: "Hideout".into(),
        }
    }

    #[test]
    fn starts_on_first_map() {
        assert_eq!(next_session_action(&in_map(), false, 0, 900), SessionAction::Start);
    }

    #[test]
    fn no_restart_while_session_active() {
        assert_eq!(next_session_action(&in_map(), true, 0, 900), SessionAction::None);
    }

    #[test]
    fn ends_at_or_after_idle_timeout() {
        assert_eq!(next_session_action(&idle(), true, 900, 900), SessionAction::End);
        assert_eq!(next_session_action(&idle(), true, 1200, 900), SessionAction::End);
    }

    #[test]
    fn no_end_before_timeout() {
        assert_eq!(next_session_action(&idle(), true, 100, 900), SessionAction::None);
    }

    #[test]
    fn no_end_without_active_session() {
        assert_eq!(next_session_action(&idle(), false, 5000, 900), SessionAction::None);
    }

    #[test]
    fn stopped_with_active_session_ends_after_timeout() {
        assert_eq!(
            next_session_action(&TrackerState::Stopped, true, 1000, 900),
            SessionAction::End
        );
    }
}
