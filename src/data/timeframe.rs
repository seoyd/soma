use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeframeSpec {
    pub timeframe: Timeframe,
    pub seconds: u32,
    pub expected_ms_step: u64,
    pub allow_gaps: bool,
    pub session_aware: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl TimeframeSpec {
    pub fn from_timeframe(timeframe: Timeframe) -> Self {
        let (seconds, reason_codes) = match timeframe {
            Timeframe::OneMinute => (60, Vec::new()),
            Timeframe::FiveMinute => (300, Vec::new()),
            Timeframe::FifteenMinute => (900, Vec::new()),
            Timeframe::OneHour => (3_600, Vec::new()),
            Timeframe::OneDay => (86_400, Vec::new()),
            Timeframe::Custom { seconds } if seconds > 0 => (seconds, Vec::new()),
            Timeframe::Custom { .. } => (0, vec![ReasonCode::UnsupportedTimeframe]),
        };
        Self {
            timeframe,
            seconds,
            expected_ms_step: seconds as u64 * 1_000,
            allow_gaps: matches!(timeframe, Timeframe::OneDay),
            session_aware: matches!(timeframe, Timeframe::OneDay),
            reason_codes,
        }
    }

    pub fn is_supported(&self) -> bool {
        self.seconds > 0
    }
}
