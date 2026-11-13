use std::fs;
use std::path::Path;

use crate::constants::default_config;
use crate::models::Action;
use crate::models::ClockTime;
use crate::models::Config;
use crate::models::Rule;

pub fn load(path: &std::path::Path) -> Result<Config, String> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            write(path, &default_config())?;
            Ok(default_config())
        }
        Err(e) => Err(format!("read config: {e}")),
        Ok(data) => {
            let cfg: Config = serde_json::from_str(&data)
                .map_err(|e| format!("parse config {}: {e}", path.display()))?;
            if cfg.rules.is_empty() {
                return Err(format!("config {}: no rules", path.display()));
            }
            Ok(cfg)
        }
    }
}

pub fn write(path: &std::path::Path, cfg: &Config) -> Result<(), String> {
    let mut data = serde_json::to_string_pretty(cfg).map_err(|e| format!("encode config: {e}"))?;
    data.push('\n');
    fs::write(path, data).map_err(|e| format!("write config: {e}"))
}

pub fn rule_times(cfg: &Config) -> (Option<ClockTime>, Option<ClockTime>) {
    let mut off = None;
    let mut on = None;
    for r in &cfg.rules {
        if off.is_none() && r.action == Action::Off {
            off = Some(r.time);
        }
        if on.is_none() && r.action == Action::On {
            on = Some(r.time);
        }
    }
    (off, on)
}

pub fn set_rule_times(path: &Path, off: ClockTime, on: ClockTime) -> Result<(), String> {
    let mut cfg = load(path)?;
    let mut saw_off = false;
    let mut saw_on = false;
    for r in &mut cfg.rules {
        if r.action == Action::Off && !saw_off {
            r.time = off;
            saw_off = true;
        }
        if r.action == Action::On && !saw_on {
            r.time = on;
            saw_on = true;
        }
    }
    if !saw_off {
        cfg.rules.push(Rule {
            action: Action::Off,
            time: off,
            days: vec![],
        });
    }
    if !saw_on {
        cfg.rules.push(Rule {
            action: Action::On,
            time: on,
            days: vec![],
        });
    }
    write(path, &cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Action;
    use crate::models::ClockTime;
    use crate::models::Day;
    use crate::models::Rule;

    fn parse(input: &str) -> Result<Config, String> {
        serde_json::from_str(input).map_err(|e| e.to_string())
    }

    #[test]
    fn parses_valid_config() {
        let cfg =
            parse(r#"{"rules":[{"action":"off","time":"23:00","days":["mon","Fri"]}]}"#).unwrap();
        assert_eq!(cfg.rules.len(), 1);
        assert_eq!(cfg.rules[0].action, Action::Off);
        assert_eq!(cfg.rules[0].time, ClockTime::new(23, 0));
        assert_eq!(cfg.rules[0].days, vec![Day::Mon, Day::Fri]);
    }

    #[test]
    fn rejects_bad_action() {
        assert!(parse(r#"{"rules":[{"action":"sleep","time":"23:00"}]}"#).is_err());
    }

    #[test]
    fn rejects_bad_time() {
        assert!(parse(r#"{"rules":[{"action":"on","time":"25:00"}]}"#).is_err());
    }

    #[test]
    fn rejects_bad_day() {
        assert!(parse(r#"{"rules":[{"action":"on","time":"08:00","days":["monday"]}]}"#).is_err());
    }

    #[test]
    fn default_config_round_trips() {
        let data = serde_json::to_string(&default_config()).unwrap();
        let cfg: Config = serde_json::from_str(&data).unwrap();
        assert_eq!(cfg.rules.len(), default_config().rules.len());
        assert_eq!(cfg.rules[0].time, default_config().rules[0].time);
    }

    #[test]
    fn default_config_serializes_like_before() {
        let cfg = Config {
            rules: vec![Rule {
                action: Action::Off,
                time: ClockTime::new(23, 0),
                days: vec![],
            }],
        };
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            r#"{"rules":[{"action":"off","time":"23:00"}]}"#
        );
    }
}
