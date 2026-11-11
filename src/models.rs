use std::fmt;
use std::time::Duration;

use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono::Timelike;
use serde::Deserialize;
use serde::Serialize;

use crate::constants::POSTPONE_OPTIONS;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    On,
    Off,
}

impl Action {
    pub fn as_str(self) -> &'static str {
        match self {
            Action::On => "on",
            Action::Off => "off",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Action {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "on" => Ok(Action::On),
            "off" => Ok(Action::Off),
            _ => Err(serde::de::Error::custom(format!(
                "action {:?}: expected \"on\" or \"off\"",
                s
            ))),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ClockTime {
    pub hour: u32,
    pub minute: u32,
}

impl ClockTime {
    pub fn new(hour: u32, minute: u32) -> Self {
        Self { hour, minute }
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

impl Serialize for ClockTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ClockTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_clock_time(&s).map_err(serde::de::Error::custom)
    }
}

fn parse_clock_time(s: &str) -> Result<ClockTime, String> {
    let invalid = || format!("time {:?}: expected HH:MM", s);
    let bytes = s.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 2 || b.is_ascii_digit())
    {
        return Err(invalid());
    }
    let hour = s[0..2].parse::<u32>().map_err(|_| invalid())?;
    let minute = s[3..5].parse::<u32>().map_err(|_| invalid())?;
    if hour > 23 || minute > 59 {
        return Err(invalid());
    }
    Ok(ClockTime::new(hour, minute))
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Day {
    pub fn name(self) -> &'static str {
        match self {
            Day::Mon => "mon",
            Day::Tue => "tue",
            Day::Wed => "wed",
            Day::Thu => "thu",
            Day::Fri => "fri",
            Day::Sat => "sat",
            Day::Sun => "sun",
        }
    }

    pub fn parse(s: &str) -> Result<Day, String> {
        match s.to_lowercase().as_str() {
            "mon" => Ok(Day::Mon),
            "tue" => Ok(Day::Tue),
            "wed" => Ok(Day::Wed),
            "thu" => Ok(Day::Thu),
            "fri" => Ok(Day::Fri),
            "sat" => Ok(Day::Sat),
            "sun" => Ok(Day::Sun),
            _ => Err(format!(
                "day {:?}: expected one of mon, tue, wed, thu, fri, sat, sun",
                s
            )),
        }
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Serialize for Day {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for Day {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Day::parse(&s).map_err(serde::de::Error::custom)
    }
}

pub fn weekday_of(at: DateTime<Local>) -> Day {
    match at.weekday() {
        chrono::Weekday::Mon => Day::Mon,
        chrono::Weekday::Tue => Day::Tue,
        chrono::Weekday::Wed => Day::Wed,
        chrono::Weekday::Thu => Day::Thu,
        chrono::Weekday::Fri => Day::Fri,
        chrono::Weekday::Sat => Day::Sat,
        chrono::Weekday::Sun => Day::Sun,
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub action: Action,
    pub time: ClockTime,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub days: Vec<Day>,
}

impl Rule {
    pub fn matches(&self, at: DateTime<Local>) -> bool {
        if at.hour() != self.time.hour || at.minute() != self.time.minute {
            return false;
        }
        self.days.is_empty() || self.days.contains(&weekday_of(at))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub rules: Vec<Rule>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FiringKey {
    pub date: String,
    pub rule: usize,
    pub warning: bool,
}

pub struct PostponeOption {
    pub label: &'static str,
    pub delay: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct PostponeRequest {
    pub off: DateTime<Local>,
    pub delay: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct DeferredOff {
    pub at: DateTime<Local>,
    pub warned: bool,
}

pub type ApplyFn = Box<dyn FnMut(Action) -> Result<(), String> + Send>;
pub type WarnFn = Arc<dyn Fn(Duration) -> Result<Duration, String> + Send + Sync>;

use std::sync::Arc;

pub fn postpone_delay(option_index: usize) -> Option<Duration> {
    POSTPONE_OPTIONS.get(option_index).map(|o| o.delay)
}

pub fn trunc_minute(t: DateTime<Local>) -> DateTime<Local> {
    t.with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .expect("zeroing wall clock fields")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_time_parses_strict_hhmm() {
        assert_eq!(parse_clock_time("23:00").unwrap(), ClockTime::new(23, 0));
        assert_eq!(parse_clock_time("08:05").unwrap(), ClockTime::new(8, 5));
        assert!(parse_clock_time("25:00").is_err());
        assert!(parse_clock_time("8:05").is_err());
        assert!(parse_clock_time("23:60").is_err());
        assert!(parse_clock_time("2300").is_err());
        assert!(parse_clock_time("23:0a").is_err());
    }

    #[test]
    fn day_parses_case_insensitive() {
        assert_eq!(Day::parse("Mon").unwrap(), Day::Mon);
        assert_eq!(Day::parse("FRI").unwrap(), Day::Fri);
        assert!(Day::parse("monday").is_err());
    }
}
