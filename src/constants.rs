use std::time::Duration;

use crate::models::Action;
use crate::models::ClockTime;
use crate::models::Config;
use crate::models::PostponeOption;
use crate::models::Rule;

pub const CONFIG_FILE_NAME: &str = "config.json";
pub const LOG_FILE_NAME: &str = "monitors.log";
pub const CHECK_INTERVAL: Duration = Duration::from_secs(15);
pub const MANUAL_OFF_DELAY: Duration = Duration::from_secs(1);
pub const WARNING_LEAD: Duration = Duration::from_secs(60);
pub const TIMER_FILE_NAME: &str = "timer.json";
pub const TIMER_TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

pub const POSTPONE_OPTIONS: [PostponeOption; 4] = [
    PostponeOption {
        label: "15 минут",
        delay: Duration::from_secs(15 * 60),
    },
    PostponeOption {
        label: "30 минут",
        delay: Duration::from_secs(30 * 60),
    },
    PostponeOption {
        label: "1 час",
        delay: Duration::from_secs(60 * 60),
    },
    PostponeOption {
        label: "2 часа",
        delay: Duration::from_secs(2 * 60 * 60),
    },
];

pub fn default_config() -> Config {
    Config {
        rules: vec![
            Rule {
                action: Action::Off,
                time: ClockTime::new(23, 0),
                days: vec![],
            },
            Rule {
                action: Action::On,
                time: ClockTime::new(8, 0),
                days: vec![],
            },
        ],
    }
}
