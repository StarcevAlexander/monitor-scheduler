use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;

use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;

use crate::config;
use crate::constants::CHECK_INTERVAL;
use crate::constants::TIMER_FILE_NAME;
use crate::constants::TIMER_TIME_FORMAT;
use crate::constants::WARNING_LEAD;
use crate::logf;
use crate::models::trunc_minute;
use crate::models::Action;
use crate::models::ApplyFn;
use crate::models::Config;
use crate::models::DeferredOff;
use crate::models::FiringKey;
use crate::models::PostponeRequest;
use crate::models::WarnFn;
use crate::schedule::due_rules;
use crate::schedule::due_warnings;
use crate::schedule::rules_for_log;

pub struct Scheduler {
    config_path: PathBuf,
    config: Config,
    mod_time: SystemTime,
    fired: HashSet<FiringKey>,
    skipped: HashSet<i64>,
    deferred: Vec<DeferredOff>,
    postpones_tx: mpsc::Sender<PostponeRequest>,
    pub(crate) postpones_rx: mpsc::Receiver<PostponeRequest>,
    apply: ApplyFn,
    warn: WarnFn,
    timer: Option<DateTime<Local>>,
    timer_warned: bool,
    timer_raw: Option<String>,
}

impl Scheduler {
    pub fn new(config_path: &Path, apply: ApplyFn, warn: WarnFn) -> Result<Self, String> {
        let (postpones_tx, postpones_rx) = mpsc::channel();
        let mut s = Scheduler {
            config_path: config_path.to_path_buf(),
            config: Config { rules: vec![] },
            mod_time: SystemTime::UNIX_EPOCH,
            fired: HashSet::new(),
            skipped: HashSet::new(),
            deferred: Vec::new(),
            postpones_tx,
            postpones_rx,
            apply,
            warn,
            timer: None,
            timer_warned: false,
            timer_raw: None,
        };
        s.reload()?;
        Ok(s)
    }

    fn reload(&mut self) -> Result<(), String> {
        let cfg = config::load(&self.config_path)?;
        let mod_time = fs_modified(&self.config_path)?;
        self.config = cfg;
        self.mod_time = mod_time;
        for line in rules_for_log(&self.config).lines() {
            logf!("{line}");
        }
        Ok(())
    }

    fn reload_if_changed(&mut self) {
        let Ok(meta) = std::fs::metadata(&self.config_path) else {
            return;
        };
        let Ok(mod_time) = meta.modified() else {
            return;
        };
        if mod_time == self.mod_time {
            return;
        }
        logf!("config changed, reloading");
        if let Err(e) = self.reload() {
            logf!("reload failed, keeping previous rules: {e}");
            self.mod_time = mod_time;
        }
    }

    pub fn tick(&mut self, now: DateTime<Local>) {
        self.reload_if_changed();
        self.drain_postpones();
        self.sync_timer();
        let today = now.format("%Y-%m-%d").to_string();
        self.fired.retain(|key| key.date == today);
        let now_unix = now.timestamp();
        self.skipped.retain(|minute| now_unix - minute < 60);
        for _ in due_warnings(&self.config.rules, now, &mut self.fired) {
            self.start_warning(trunc_minute(now + WARNING_LEAD), now, false);
        }
        let mut to_warn = Vec::new();
        for d in &mut self.deferred {
            if !d.warned && now < d.at && now >= d.at - WARNING_LEAD {
                d.warned = true;
                to_warn.push(d.at);
            }
        }
        for at in to_warn {
            self.start_warning(at, now, false);
        }
        for r in due_rules(&self.config.rules, now, &mut self.fired) {
            if r.action == Action::Off && self.skipped.contains(&trunc_minute(now).timestamp()) {
                logf!("monitors off ({}) postponed", r.time);
                continue;
            }
            self.run_action(r.action, r.time.to_string());
        }
        let mut pending = Vec::new();
        for d in std::mem::take(&mut self.deferred) {
            if now < d.at {
                pending.push(d);
                continue;
            }
            self.run_action(Action::Off, d.at.format("%H:%M").to_string());
        }
        self.deferred = pending;
        if let Some(at) = self.timer {
            if now >= at {
                self.run_action(Action::Off, "timer".to_string());
                self.clear_timer_file();
            } else if !self.timer_warned && now >= at - WARNING_LEAD {
                self.timer_warned = true;
                self.start_warning(at, now, true);
            }
        }
    }

    fn run_action(&mut self, a: Action, at: String) {
        match (self.apply)(a) {
            Err(e) => logf!("{a} at {at} failed: {e}"),
            Ok(()) => logf!("monitors {a} ({at})"),
        }
    }

    fn start_warning(&self, off: DateTime<Local>, now: DateTime<Local>, is_timer: bool) {
        logf!("warning: monitors off at {}", off.format("%H:%M"));
        let warn = Arc::clone(&self.warn);
        let tx = self.postpones_tx.clone();
        std::thread::spawn(move || {
            let timeout = (off - now).to_std().unwrap_or(Duration::ZERO);
            match warn(timeout) {
                Err(e) => logf!("warning before {} failed: {e}", off.format("%H:%M")),
                Ok(delay) => {
                    if delay > Duration::ZERO {
                        let _ = tx.send(PostponeRequest {
                            off,
                            delay,
                            is_timer,
                        });
                    }
                }
            }
        });
    }

    fn drain_postpones(&mut self) {
        while let Ok(req) = self.postpones_rx.try_recv() {
            self.postpone(req);
        }
    }

    fn postpone(&mut self, req: PostponeRequest) {
        if req.is_timer {
            if self.timer != Some(req.off) {
                return;
            }
            let at = req.off
                + chrono::Duration::from_std(req.delay).expect("postpone delay out of range");
            if let Some(path) = self.timer_path() {
                if let Err(e) = write_timer_file(&path, at) {
                    logf!("timer file write failed: {e}");
                }
            }
            self.timer = Some(at);
            self.timer_warned = false;
            self.timer_raw = None;
            logf!("timer postponed to {}", at.format(TIMER_TIME_FORMAT));
            return;
        }
        self.skipped.insert(trunc_minute(req.off).timestamp());
        self.deferred.retain(|d| d.at != req.off);
        let at =
            req.off + chrono::Duration::from_std(req.delay).expect("postpone delay out of range");
        self.deferred.push(DeferredOff { at, warned: false });
        logf!(
            "monitors off at {} postponed to {}",
            req.off.format("%H:%M"),
            at.format("%H:%M")
        );
    }

    fn timer_path(&self) -> Option<PathBuf> {
        self.config_path
            .parent()
            .map(|dir| dir.join(TIMER_FILE_NAME))
    }

    fn sync_timer(&mut self) {
        let Some(path) = self.timer_path() else {
            return;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            if self.timer.take().is_some() || self.timer_warned {
                self.timer_warned = false;
                logf!("timer cleared");
            }
            self.timer_raw = None;
            return;
        };
        if self.timer_raw.as_deref() == Some(text.as_str()) {
            return;
        }
        self.timer_raw = Some(text.clone());
        match parse_timer_file(&text) {
            Some(at) => {
                if self.timer != Some(at) {
                    logf!(
                        "timer set: monitors off at {}",
                        at.format(TIMER_TIME_FORMAT)
                    );
                    self.timer = Some(at);
                    self.timer_warned = false;
                }
            }
            None => logf!("timer file invalid, ignoring: {}", text.trim()),
        }
    }

    fn clear_timer_file(&mut self) {
        if let Some(path) = self.timer_path() {
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => logf!("timer file remove failed: {e}"),
            }
        }
        self.timer = None;
        self.timer_warned = false;
        self.timer_raw = None;
    }

    pub fn run(&mut self) {
        self.tick(Local::now());
        loop {
            let deadline = Instant::now() + CHECK_INTERVAL;
            loop {
                let now = Instant::now();
                if now >= deadline {
                    break;
                }
                match self.postpones_rx.recv_timeout(deadline - now) {
                    Ok(req) => self.postpone(req),
                    Err(mpsc::RecvTimeoutError::Timeout) => break,
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                    }
                }
            }
            self.tick(Local::now());
        }
    }
}

fn fs_modified(path: &Path) -> Result<SystemTime, String> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map_err(|e| format!("stat config: {e}"))
}

pub fn parse_timer_file(text: &str) -> Option<DateTime<Local>> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let raw = value.get("off_at")?.as_str()?;
    let naive = chrono::NaiveDateTime::parse_from_str(raw, TIMER_TIME_FORMAT).ok()?;
    Local.from_local_datetime(&naive).single()
}

pub fn write_timer_file(path: &Path, at: DateTime<Local>) -> Result<(), String> {
    let body = format!("{{\"off_at\": \"{}\"}}\n", at.format(TIMER_TIME_FORMAT));
    std::fs::write(path, body).map_err(|e| format!("write timer file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ClockTime;
    use crate::models::Rule;
    use chrono::TimeZone;
    use std::sync::Mutex;

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(year, month, day, hour, minute, second)
            .unwrap()
    }

    struct TestRig {
        scheduler: Scheduler,
        applied_rx: mpsc::Receiver<Action>,
        answers_tx: mpsc::Sender<Duration>,
    }

    fn new_test_scheduler(rules: Vec<Rule>) -> TestRig {
        let (applied_tx, applied_rx) = mpsc::channel();
        let (answers_tx, answers_rx) = mpsc::channel();
        let answers = Arc::new(Mutex::new(answers_rx));
        let (postpones_tx, postpones_rx) = mpsc::channel();
        let scheduler = Scheduler {
            config_path: PathBuf::new(),
            config: Config { rules },
            mod_time: SystemTime::UNIX_EPOCH,
            fired: HashSet::new(),
            skipped: HashSet::new(),
            deferred: Vec::new(),
            postpones_tx,
            postpones_rx,
            apply: Box::new(move |a| {
                applied_tx.send(a).map_err(|e| e.to_string())?;
                Ok(())
            }),
            warn: Arc::new(move |_| {
                answers
                    .lock()
                    .expect("answers mutex poisoned")
                    .recv()
                    .map_err(|e| e.to_string())
            }),
            timer: None,
            timer_warned: false,
            timer_raw: None,
        };
        TestRig {
            scheduler,
            applied_rx,
            answers_tx,
        }
    }

    fn receive_postpone(rig: &TestRig) -> PostponeRequest {
        rig.scheduler
            .postpones_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("no postpone request")
    }

    fn applied(rig: &TestRig) -> Vec<Action> {
        let mut out = Vec::new();
        while let Ok(a) = rig.applied_rx.try_recv() {
            out.push(a);
        }
        out
    }

    #[test]
    fn scheduler_postpones_off() {
        let mut rig = new_test_scheduler(vec![Rule {
            action: Action::Off,
            time: ClockTime::new(23, 0),
            days: vec![],
        }]);
        let base = at(2026, 10, 26, 22, 59, 10);

        rig.answers_tx.send(Duration::from_secs(30 * 60)).unwrap();
        rig.scheduler.tick(base);
        let req = receive_postpone(&rig);
        assert_eq!(req.off, trunc_minute(base + WARNING_LEAD));
        assert_eq!(req.delay, Duration::from_secs(30 * 60));
        rig.scheduler.postpone(req);

        rig.scheduler.tick(base + chrono::Duration::seconds(55));
        assert!(
            applied(&rig).is_empty(),
            "postponed off still applied at 23:00"
        );

        rig.answers_tx.send(Duration::ZERO).unwrap();
        rig.scheduler.tick(at(2026, 10, 26, 23, 29, 5));
        assert!(rig
            .scheduler
            .postpones_rx
            .recv_timeout(Duration::from_millis(50))
            .is_err());
        assert!(applied(&rig).is_empty(), "applied before deferred time");

        rig.scheduler.tick(at(2026, 10, 26, 23, 30, 5));
        assert_eq!(applied(&rig), vec![Action::Off]);
        rig.scheduler.tick(at(2026, 10, 26, 23, 30, 20));
        assert!(applied(&rig).is_empty(), "deferred off applied twice");
    }

    #[test]
    fn scheduler_postpones_deferred_off_again() {
        let mut rig = new_test_scheduler(vec![]);
        let off = at(2026, 10, 26, 23, 30, 0);
        rig.scheduler.deferred = vec![DeferredOff {
            at: off,
            warned: false,
        }];

        rig.answers_tx.send(Duration::from_secs(15 * 60)).unwrap();
        rig.scheduler.tick(off - chrono::Duration::seconds(50));
        rig.scheduler.postpone(receive_postpone(&rig));

        rig.scheduler.tick(off + chrono::Duration::seconds(5));
        assert!(
            applied(&rig).is_empty(),
            "re-postponed off applied at 23:30"
        );
        assert_eq!(rig.scheduler.deferred.len(), 1);
        assert_eq!(
            rig.scheduler.deferred[0].at,
            off + chrono::Duration::minutes(15)
        );
    }

    #[test]
    fn scheduler_on_rule_not_skipped() {
        let mut rig = new_test_scheduler(vec![Rule {
            action: Action::On,
            time: ClockTime::new(23, 0),
            days: vec![],
        }]);
        let t = at(2026, 10, 26, 23, 0, 5);
        rig.scheduler.skipped.insert(trunc_minute(t).timestamp());
        rig.scheduler.tick(t);
        assert_eq!(applied(&rig), vec![Action::On]);
    }

    fn timer_test_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("monitors-test-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn timer_fires_when_due_and_deletes_file() {
        let dir = timer_test_dir("fire");
        let mut rig = new_test_scheduler(vec![]);
        rig.scheduler.config_path = dir.join("config.json");
        let fire_at = at(2026, 11, 13, 23, 45, 0);
        write_timer_file(&dir.join(crate::constants::TIMER_FILE_NAME), fire_at).unwrap();

        rig.scheduler.tick(fire_at - chrono::Duration::seconds(30));
        assert!(applied(&rig).is_empty(), "fired before due time");
        assert_eq!(rig.scheduler.timer, Some(fire_at));
        assert!(rig.scheduler.timer_warned, "warning must start before fire");

        rig.scheduler.tick(fire_at + chrono::Duration::seconds(5));
        assert_eq!(applied(&rig), vec![Action::Off]);
        assert_eq!(rig.scheduler.timer, None);
        assert!(
            !dir.join(crate::constants::TIMER_FILE_NAME).exists(),
            "timer file must be deleted after fire"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn timer_cancelled_when_file_removed() {
        let dir = timer_test_dir("cancel");
        let mut rig = new_test_scheduler(vec![]);
        rig.scheduler.config_path = dir.join("config.json");
        let fire_at = at(2026, 11, 13, 23, 45, 0);
        write_timer_file(&dir.join(crate::constants::TIMER_FILE_NAME), fire_at).unwrap();
        rig.scheduler.tick(fire_at - chrono::Duration::seconds(30));
        assert_eq!(rig.scheduler.timer, Some(fire_at));

        std::fs::remove_file(dir.join(crate::constants::TIMER_FILE_NAME)).unwrap();
        rig.scheduler.tick(fire_at - chrono::Duration::seconds(20));
        assert_eq!(rig.scheduler.timer, None, "cancel via file removal");
        assert!(applied(&rig).is_empty(), "cancelled timer must not fire");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn timer_postpone_rewrites_file() {
        let dir = timer_test_dir("postpone");
        let mut rig = new_test_scheduler(vec![]);
        rig.scheduler.config_path = dir.join("config.json");
        let fire_at = at(2026, 11, 13, 23, 45, 0);
        write_timer_file(&dir.join(crate::constants::TIMER_FILE_NAME), fire_at).unwrap();
        rig.scheduler.tick(fire_at - chrono::Duration::seconds(30));
        rig.answers_tx.send(Duration::from_secs(30 * 60)).unwrap();

        rig.scheduler.tick(fire_at - chrono::Duration::seconds(20));
        let req = receive_postpone(&rig);
        assert!(req.is_timer, "timer warning must be marked as timer");
        rig.scheduler.postpone(req);

        let new_at = fire_at + chrono::Duration::minutes(30);
        assert_eq!(rig.scheduler.timer, Some(new_at));
        let saved = std::fs::read_to_string(dir.join(crate::constants::TIMER_FILE_NAME)).unwrap();
        assert_eq!(
            parse_timer_file(&saved),
            Some(new_at),
            "file must hold postponed time"
        );

        rig.scheduler.tick(new_at + chrono::Duration::seconds(5));
        assert_eq!(applied(&rig), vec![Action::Off]);
        std::fs::remove_dir_all(&dir).ok();
    }
}
