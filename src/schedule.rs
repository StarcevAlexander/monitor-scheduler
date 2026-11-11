use std::collections::HashSet;

use chrono::DateTime;
use chrono::Local;

use crate::constants::WARNING_LEAD;
use crate::models::Action;
use crate::models::Config;
use crate::models::FiringKey;
use crate::models::Rule;

pub fn due_rules(
    rules: &[Rule],
    now: DateTime<Local>,
    fired: &mut HashSet<FiringKey>,
) -> Vec<Rule> {
    collect_due(rules, now, now, false, fired)
}

pub fn due_warnings(
    rules: &[Rule],
    now: DateTime<Local>,
    fired: &mut HashSet<FiringKey>,
) -> Vec<Rule> {
    collect_due(rules, now, now + WARNING_LEAD, true, fired)
}

fn collect_due(
    rules: &[Rule],
    now: DateTime<Local>,
    at: DateTime<Local>,
    warning: bool,
    fired: &mut HashSet<FiringKey>,
) -> Vec<Rule> {
    let date = now.format("%Y-%m-%d").to_string();
    let mut due = Vec::new();
    for (i, r) in rules.iter().enumerate() {
        if (warning && r.action != Action::Off) || !r.matches(at) {
            continue;
        }
        let key = FiringKey {
            date: date.clone(),
            rule: i,
            warning,
        };
        if !fired.insert(key) {
            continue;
        }
        due.push(r.clone());
    }
    due
}

pub fn rules_for_log(config: &Config) -> String {
    config
        .rules
        .iter()
        .map(|r| {
            let days: Vec<String> = r.days.iter().map(|d| d.to_string()).collect();
            format!("rule: {} at {} days=[{}]", r.action, r.time, days.join(" "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ClockTime;
    use crate::models::Day;
    use chrono::TimeZone;

    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap()
    }

    #[test]
    fn rule_matches() {
        let monday = at(2026, 10, 26, 23, 0, 30);
        let cases = [
            (
                "every day exact minute",
                Rule {
                    action: Action::Off,
                    time: ClockTime::new(23, 0),
                    days: vec![],
                },
                monday,
                true,
            ),
            (
                "other minute",
                Rule {
                    action: Action::Off,
                    time: ClockTime::new(23, 1),
                    days: vec![],
                },
                monday,
                false,
            ),
            (
                "weekday listed",
                Rule {
                    action: Action::Off,
                    time: ClockTime::new(23, 0),
                    days: vec![Day::Mon],
                },
                monday,
                true,
            ),
            (
                "weekday not listed",
                Rule {
                    action: Action::Off,
                    time: ClockTime::new(23, 0),
                    days: vec![Day::Sun],
                },
                monday,
                false,
            ),
        ];
        for (name, rule, time, want) in cases {
            assert_eq!(rule.matches(time), want, "{name}");
        }
    }

    #[test]
    fn due_rules_fires_once_per_day() {
        let rules = [Rule {
            action: Action::Off,
            time: ClockTime::new(23, 0),
            days: vec![],
        }];
        let mut fired = HashSet::new();
        let first = at(2026, 10, 26, 23, 0, 0);
        assert_eq!(due_rules(&rules, first, &mut fired).len(), 1);
        assert_eq!(
            due_rules(&rules, first + chrono::Duration::seconds(30), &mut fired).len(),
            0
        );
        let next_day = Local.with_ymd_and_hms(2026, 10, 27, 23, 0, 0).unwrap();
        assert_eq!(due_rules(&rules, next_day, &mut fired).len(), 1);
    }

    #[test]
    fn due_warnings_lead_one_minute() {
        let rules = [
            Rule {
                action: Action::Off,
                time: ClockTime::new(23, 0),
                days: vec![],
            },
            Rule {
                action: Action::On,
                time: ClockTime::new(23, 0),
                days: vec![],
            },
        ];
        let mut fired = HashSet::new();
        let before = at(2026, 10, 26, 22, 59, 0);
        assert_eq!(
            due_warnings(&rules, before - chrono::Duration::minutes(1), &mut fired).len(),
            0
        );
        let got = due_warnings(&rules, before, &mut fired);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].action, Action::Off);
        assert_eq!(
            due_warnings(&rules, before + chrono::Duration::seconds(30), &mut fired).len(),
            0
        );
        assert_eq!(
            due_rules(&rules, before + chrono::Duration::minutes(1), &mut fired).len(),
            2
        );
    }

    #[test]
    fn due_warnings_across_midnight() {
        let rules = [Rule {
            action: Action::Off,
            time: ClockTime::new(0, 0),
            days: vec![Day::Mon],
        }];
        let sunday = at(2026, 10, 25, 23, 59, 10);
        assert_eq!(due_warnings(&rules, sunday, &mut HashSet::new()).len(), 1);
        let saturday = sunday - chrono::Duration::days(1);
        assert_eq!(due_warnings(&rules, saturday, &mut HashSet::new()).len(), 0);
    }
}
