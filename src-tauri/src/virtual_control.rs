//! Development-only, in-memory virtual model. No transport or vehicle references.
//! The orchestration layer exposes development-only IPC and holds the interlock.
use std::time::{Duration, Instant};
const INPUT_TTL: Duration = Duration::from_millis(500);
const MAX_STEP: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Axes {
    pub forward: f64,
    pub right: f64,
    pub up: f64,
    pub yaw: f64,
}
impl Axes {
    fn valid(self) -> bool {
        [self.forward, self.right, self.up, self.yaw]
            .iter()
            .all(|x| x.is_finite() && x.abs() <= 1.0)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VirtualState {
    pub north: f64,
    pub east: f64,
    pub up: f64,
    pub yaw_deg: f64,
    pub velocity: Axes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    Offline,
    Ready,
    Control,
    Locked,
}
#[derive(Debug)]
pub struct VirtualControl {
    authority: Authority,
    generation: u64,
    input: Axes,
    updated: Option<Instant>,
    last_sequence: Option<u64>,
    state: VirtualState,
}
impl Default for VirtualControl {
    fn default() -> Self {
        Self {
            authority: Authority::Offline,
            generation: 0,
            input: Axes::default(),
            updated: None,
            last_sequence: None,
            state: VirtualState::default(),
        }
    }
}
impl VirtualControl {
    pub fn start(&mut self) -> u64 {
        self.generation = self.generation.saturating_add(1);
        self.authority = Authority::Ready;
        self.input = Axes::default();
        self.updated = None;
        self.last_sequence = None;
        self.state = VirtualState::default();
        self.generation
    }
    pub fn acquire(&mut self, session: u64) -> bool {
        if session != self.generation || self.authority != Authority::Ready {
            return false;
        }
        self.authority = Authority::Control;
        true
    }
    fn input(&mut self, session: u64, input: Axes, at: Instant) -> bool {
        if session != self.generation || self.authority != Authority::Control {
            return false;
        }
        if !input.valid() || self.updated.is_some_and(|old| at < old) {
            self.lock();
            return false;
        }
        self.input = input;
        self.updated = Some(at);
        true
    }
    /// Reject delayed or duplicated requests before they can restore old axes.
    pub fn input_ordered(&mut self, session: u64, sequence: u64, input: Axes, at: Instant) -> bool {
        if session != self.generation
            || self.authority != Authority::Control
            || self.last_sequence.is_some_and(|last| sequence <= last)
        {
            return false;
        }
        // A delayed input must not revive control after its lease expired,
        // even when no status poll occurred during the pause.
        if self.updated.is_some_and(|last| {
            at.checked_duration_since(last)
                .is_none_or(|age| age >= INPUT_TTL)
        }) {
            self.lock();
            return false;
        }
        if !self.input(session, input, at) {
            return false;
        }
        self.last_sequence = Some(sequence);
        true
    }
    pub fn advance(&mut self, now: Instant, step: Duration) -> VirtualState {
        if self.authority != Authority::Control
            || self.updated.is_none_or(|at| {
                now.checked_duration_since(at)
                    .is_none_or(|age| age >= INPUT_TTL)
            })
        {
            if self.authority == Authority::Control {
                self.lock();
            }
            self.state.velocity = Axes::default();
            return self.state;
        }
        let dt = step.min(MAX_STEP).as_secs_f64();
        self.state.north += self.input.forward * dt;
        self.state.east += self.input.right * dt;
        self.state.up += self.input.up * dt;
        self.state.yaw_deg = (self.state.yaw_deg + self.input.yaw * 45.0 * dt).rem_euclid(360.0);
        self.state.velocity = self.input;
        self.state
    }
    pub fn lock(&mut self) {
        self.authority = Authority::Locked;
        self.input = Axes::default();
        self.updated = None;
        self.state.velocity = Axes::default();
    }
    pub fn stop(&mut self) {
        self.lock();
        self.authority = Authority::Offline;
    }
    pub fn authority(&self) -> Authority {
        self.authority
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_timeout_and_invalid_inputs_fail_closed() {
        let t = Instant::now();
        let mut v = VirtualControl::default();
        let first = v.start();
        assert!(v.acquire(first));
        assert!(v.input(
            first,
            Axes {
                forward: 1.0,
                yaw: 1.0,
                ..Axes::default()
            },
            t
        ));
        let s = v.advance(t, Duration::from_secs(1));
        assert_eq!(s.north, 0.1);
        assert_eq!(s.yaw_deg, 4.5);
        let s = v.advance(t + INPUT_TTL, Duration::from_millis(10));
        assert_eq!(v.authority(), Authority::Locked);
        assert_eq!(s.velocity, Axes::default());
        assert!(!v.input(first, Axes::default(), t + INPUT_TTL));
        let next = v.start();
        assert_ne!(next, first);
        assert!(v.acquire(next));
        assert!(!v.input(first, Axes::default(), t));
        assert!(!v.input(
            next,
            Axes {
                up: f64::NAN,
                ..Axes::default()
            },
            t
        ));
        assert_eq!(v.authority(), Authority::Locked);
        let next = v.start();
        assert!(v.acquire(next));
        assert!(v.input(next, Axes::default(), t));
        assert!(!v.input(next, Axes::default(), t - Duration::from_millis(1)));
        assert_eq!(v.authority(), Authority::Locked);
        v.stop();
        assert_eq!(v.authority(), Authority::Offline);
    }
    #[test]
    fn neutral_lock_and_new_session_clear_virtual_motion() {
        let now = Instant::now();
        let mut v = VirtualControl::default();
        let session = v.start();
        assert!(v.acquire(session));
        assert!(v.input(
            session,
            Axes {
                right: -1.0,
                up: 1.0,
                ..Axes::default()
            },
            now
        ));
        let moved = v.advance(now, Duration::from_millis(100));
        assert_eq!((moved.east, moved.up), (-0.1, 0.1));
        assert!(v.input(session, Axes::default(), now));
        assert_eq!(
            v.advance(now, Duration::from_millis(100)),
            VirtualState {
                velocity: Axes::default(),
                ..moved
            }
        );
        v.lock();
        assert!(!v.acquire(session));
        assert_eq!(
            v.advance(now, Duration::from_millis(100)).velocity,
            Axes::default()
        );
        let new_session = v.start();
        assert_ne!(new_session, session);
        assert_eq!(
            v.advance(now, Duration::from_millis(100)),
            VirtualState::default()
        );
        assert!(!v.input(
            session,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            now
        ));
        assert_eq!(v.authority(), Authority::Ready);
    }
    #[test]
    fn late_and_duplicate_commands_do_not_restore_previous_input() {
        let now = Instant::now();
        let mut v = VirtualControl::default();
        let id = v.start();
        assert!(v.acquire(id));
        assert!(v.input_ordered(
            id,
            1,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            now
        ));
        assert!(v.input_ordered(id, 2, Axes::default(), now));
        assert!(!v.input_ordered(
            id,
            1,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            now
        ));
        assert!(!v.input_ordered(
            id,
            2,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            now
        ));
        assert_eq!(
            v.advance(now, Duration::from_millis(100)),
            VirtualState::default()
        );
    }
    #[test]
    fn late_input_cannot_revive_an_expired_lease_without_a_poll() {
        let now = Instant::now();
        let mut v = VirtualControl::default();
        let id = v.start();
        assert!(v.acquire(id));
        assert!(v.input_ordered(id, 1, Axes::default(), now));
        assert!(!v.input_ordered(
            id,
            2,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            now + INPUT_TTL
        ));
        assert_eq!(v.authority(), Authority::Locked);
        assert_eq!(
            v.advance(now + INPUT_TTL, Duration::from_millis(100))
                .velocity,
            Axes::default()
        );
    }
}

// IPC orchestration is separate from the in-memory model: only this layer
// receives the connection manager, and it never supplies transport to the model.
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
#[cfg(test)]
fn append_journal_at(dir: &std::path::Path, ticket: u64, code: &str) -> Result<(), String> {
    append_event_at(
        dir,
        JournalEvent {
            ticket,
            code: code.into(),
            reason: None,
            time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        },
    )
}
fn append_event_at(dir: &std::path::Path, event: JournalEvent) -> Result<(), String> {
    use std::io::Write;
    std::fs::create_dir_all(dir).map_err(|_| "가상 이벤트 디렉터리 오류")?;
    let mut options = std::fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(dir.join("virtual-control-events.jsonl"))
        .map_err(|_| "가상 이벤트 기록 열기 오류")?;
    if event.code.is_empty() {
        return Ok(());
    }
    let line = serde_json::json!({"version":1,"type":"event","source":"VIRTUAL_MEMORY","session":event.ticket,"time_ms":event.time_ms,"code":event.code,"reason":event.reason});
    writeln!(file, "{line}").map_err(|_| "가상 이벤트 기록 오류".into())
}
struct JournalEvent {
    ticket: u64,
    code: String,
    time_ms: u64,
    reason: Option<String>,
}
struct Journal {
    sender: mpsc::SyncSender<JournalEvent>,
    failed: Arc<AtomicBool>,
    pending: Arc<AtomicUsize>,
}
impl Journal {
    fn new<F>(capacity: usize, mut write: F) -> Self
    where
        F: FnMut(JournalEvent) -> Result<(), String> + Send + 'static,
    {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        let failed = Arc::new(AtomicBool::new(false));
        let pending = Arc::new(AtomicUsize::new(0));
        let worker_failed = failed.clone();
        let worker_pending = pending.clone();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                while let Ok(event) = receiver.recv() {
                    if write(event).is_err() {
                        worker_failed.store(true, Ordering::SeqCst);
                    }
                    worker_pending.fetch_sub(1, Ordering::SeqCst);
                }
            }));
            if result.is_err() {
                worker_failed.store(true, Ordering::SeqCst);
            }
        });
        Self {
            sender,
            failed,
            pending,
        }
    }
    fn healthy(&self) -> bool {
        !self.failed.load(Ordering::SeqCst)
    }
    fn send(&self, ticket: u64, code: &str) -> Result<(), String> {
        self.send_reason(ticket, code, None)
    }
    fn send_reason(&self, ticket: u64, code: &str, reason: Option<&str>) -> Result<(), String> {
        if !self.healthy() {
            return Err("가상 이벤트 기록 오류".into());
        }
        let event = JournalEvent {
            ticket,
            code: code.into(),
            reason: reason.map(str::to_owned),
            time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        self.pending.fetch_add(1, Ordering::SeqCst);
        if self.sender.try_send(event).is_err() {
            self.pending.fetch_sub(1, Ordering::SeqCst);
            self.failed.store(true, Ordering::SeqCst);
            return Err("가상 이벤트 큐 오류 · 기록 누락".into());
        }
        Ok(())
    }
}
fn journal(app: &tauri::AppHandle, ticket: u64, code: &str) -> Result<(), String> {
    use tauri::Manager as _;
    let manager = app.state::<VirtualManager>();
    manager
        .journal
        .get()
        .ok_or("가상 이벤트 기록 미준비")?
        .send(ticket, code)
}
#[cfg(test)]
mod journal_tests {
    use super::*;
    #[test]
    fn measure_watchdog_normal_and_stalled_writer() {
        for stalled in [false, true] {
            let mut observed = Vec::new();
            for _ in 0..3 {
                let (entered_tx, entered_rx) = mpsc::channel();
                let (unblock_tx, unblock_rx) = mpsc::channel::<()>();
                let log = Journal::new(16, move |_| {
                    let _ = entered_tx.send(());
                    if stalled {
                        let _ = unblock_rx.recv();
                    }
                    Ok(())
                });
                let manager = VirtualManager::default();
                manager.journal.set(log).ok().unwrap();
                let connection = crate::connection::Manager::default();
                let ticket = connection.claim_virtual().unwrap();
                let start = Instant::now();
                {
                    let mut runtime = manager.inner.lock().unwrap();
                    let session = runtime.model.start();
                    assert!(runtime.model.acquire(session));
                    assert!(runtime.model.input_ordered(
                        session,
                        1,
                        Axes {
                            forward: 1.0,
                            ..Axes::default()
                        },
                        start
                    ));
                    runtime.ticket = Some(ticket);
                }
                manager
                    .journal
                    .get()
                    .unwrap()
                    .send(ticket, "virtual_started")
                    .unwrap();
                entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
                loop {
                    std::thread::sleep(Duration::from_millis(50));
                    watchdog_tick(&manager, &connection);
                    if manager.inner.lock().unwrap().ticket.is_none() {
                        break;
                    }
                    assert!(start.elapsed() < Duration::from_secs(3));
                }
                observed.push(start.elapsed().as_secs_f64() * 1000.0);
                assert!(connection.claim_replay().is_ok());
                drop(unblock_tx);
            }
            observed.sort_by(f64::total_cmp);
            println!(
                "watchdog stalled={stalled} elapsed_ms={observed:?} median_ms={:.3}",
                observed[1]
            );
        }
    }
    #[test]
    fn simulated_safety_recovery_does_not_reactivate_old_input() {
        for fault in [
            "connection_loss",
            "battery_low",
            "gps_loss",
            "attitude_loss",
            "sensor_stale",
        ] {
            let mut runtime = Runtime::default();
            runtime.reset_simulated_safety();
            assert_eq!(runtime.safety_state(), "NORMAL");
            let old = runtime.model.start();
            assert!(runtime.model.acquire(old));
            let now = Instant::now();
            assert!(runtime.model.input_ordered(
                old,
                1,
                Axes {
                    forward: 1.0,
                    ..Axes::default()
                },
                now
            ));
            match fault {
                "connection_loss" => runtime.simulated_connection = false,
                "battery_low" => {
                    runtime.simulated.fields.get_mut("battery").unwrap().value = Some(10.0)
                }
                "gps_loss" => runtime.simulated.fields.get_mut("gps_fix").unwrap().status = "STALE",
                "attitude_loss" => {
                    runtime.simulated.fields.get_mut("roll").unwrap().status = "STALE"
                }
                "sensor_stale" => {
                    runtime
                        .simulated
                        .fields
                        .get_mut("sensors_health")
                        .unwrap()
                        .status = "STALE"
                }
                _ => unreachable!(),
            }
            assert_ne!(runtime.safety_state(), "NORMAL", "{fault}");
            runtime.model.lock();
            runtime.reset_simulated_safety();
            assert_eq!(runtime.safety_state(), "NORMAL", "{fault}");
            assert_eq!(runtime.model.authority(), Authority::Locked);
            assert_eq!(runtime.model.input, Axes::default());
            assert!(!runtime.model.input_ordered(
                old,
                2,
                Axes {
                    forward: 1.0,
                    ..Axes::default()
                },
                now
            ));
            runtime.model.stop();
            let new = runtime.model.start();
            assert_ne!(old, new);
            assert!(runtime.model.acquire(new));
            assert!(!runtime.model.input_ordered(
                old,
                3,
                Axes {
                    forward: 1.0,
                    ..Axes::default()
                },
                now
            ));
            assert!(runtime.model.input_ordered(new, 1, Axes::default(), now));
        }
    }
    #[test]
    fn full_queue_watchdog_and_stop_race_never_release_new_session() {
        for _ in 0..20 {
            let (entered_tx, entered_rx) = mpsc::channel();
            let (unblock_tx, unblock_rx) = mpsc::channel::<()>();
            let log = Journal::new(128, move |_| {
                let _ = entered_tx.send(());
                let _ = unblock_rx.recv();
                Ok(())
            });
            log.send(0, "block").unwrap();
            entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            for _ in 0..128 {
                log.send(0, "queued").unwrap();
            }
            assert!(log.send(0, "overflow").is_err());
            let manager = Arc::new(VirtualManager::default());
            manager.journal.set(log).ok().unwrap();
            let connection = Arc::new(crate::connection::Manager::default());
            let ticket = connection.claim_virtual().unwrap();
            {
                let mut runtime = manager.inner.lock().unwrap();
                let session = runtime.model.start();
                assert!(runtime.model.acquire(session));
                assert!(runtime.model.input_ordered(
                    session,
                    1,
                    Axes {
                        forward: 1.0,
                        ..Axes::default()
                    },
                    Instant::now()
                ));
                runtime.ticket = Some(ticket);
            }
            let barrier = Arc::new(std::sync::Barrier::new(3));
            let (m, c, b) = (manager.clone(), connection.clone(), barrier.clone());
            let watch = std::thread::spawn(move || {
                b.wait();
                watchdog_tick(&m, &c);
            });
            let (m, c, b) = (manager.clone(), connection.clone(), barrier.clone());
            let stop = std::thread::spawn(move || {
                b.wait();
                let mut r = m.inner.lock().unwrap();
                if r.ticket == Some(ticket) {
                    r.revoke(&c)
                } else {
                    None
                }
            });
            barrier.wait();
            watch.join().unwrap();
            let _ = stop.join().unwrap();
            let runtime = manager.inner.lock().unwrap();
            assert!(runtime.ticket.is_none());
            assert_eq!(runtime.model.input, Axes::default());
            assert_eq!(runtime.model.state.velocity, Axes::default());
            drop(runtime);
            let newer = connection.claim_virtual().unwrap();
            assert!(!connection.release_virtual(ticket));
            assert!(connection.claim_replay().is_err());
            assert!(connection.release_virtual(newer));
            drop(unblock_tx);
        }
    }
    #[test]
    fn watchdog_releases_active_session_while_writer_is_stalled() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (unblock_tx, unblock_rx) = mpsc::channel::<()>();
        let log = Journal::new(16, move |_| {
            let _ = entered_tx.send(());
            let _ = unblock_rx.recv();
            Ok(())
        });
        let manager = Arc::new(VirtualManager::default());
        manager.journal.set(log).ok().unwrap();
        let connection = Arc::new(crate::connection::Manager::default());
        let ticket = connection.claim_virtual().unwrap();
        let start = Instant::now();
        {
            let mut runtime = manager.inner.lock().unwrap();
            let session = runtime.model.start();
            assert!(runtime.model.acquire(session));
            assert!(runtime.model.input_ordered(
                session,
                1,
                Axes {
                    forward: 1.0,
                    ..Axes::default()
                },
                start
            ));
            runtime.model.advance(start, Duration::ZERO);
            runtime.ticket = Some(ticket);
        }
        manager
            .journal
            .get()
            .unwrap()
            .send(ticket, "virtual_started")
            .unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let m = manager.clone();
        let c = connection.clone();
        let (done_tx, done_rx) = mpsc::channel();
        let watcher = std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(50));
            watchdog_tick(&m, &c);
            if m.inner.lock().unwrap().ticket.is_none() {
                done_tx.send(Instant::now()).unwrap();
                break;
            }
        });
        let elapsed = done_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .duration_since(start);
        assert!(
            elapsed >= INPUT_TTL && elapsed < Duration::from_secs(2),
            "{elapsed:?}"
        );
        let runtime = manager.inner.lock().unwrap();
        assert_eq!(runtime.model.input, Axes::default());
        assert_eq!(runtime.model.state.velocity, Axes::default());
        assert_eq!(runtime.model.authority(), Authority::Offline);
        drop(runtime);
        let replay = connection.claim_replay().unwrap();
        assert!(connection.release_replay(replay));
        assert!(!connection.release_virtual(ticket));
        // Writer has remained blocked throughout the independent observation.
        watcher.join().unwrap();
        drop(unblock_tx);
    }
    #[test]
    fn blocked_writer_and_full_queue_do_not_block_revoke() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (unblock_tx, unblock_rx) = mpsc::channel();
        let log = Journal::new(1, move |_| {
            let _ = entered_tx.send(());
            let _ = unblock_rx.recv();
            Ok(())
        });
        log.send(1, "virtual_started").unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        log.send(1, "authority_granted").unwrap();
        let started = Instant::now();
        assert!(log.send(1, "input_reset").is_err());
        assert!(!log.healthy());
        let connection = crate::connection::Manager::default();
        let mut runtime = Runtime::default();
        let ticket = connection.claim_virtual().unwrap();
        let session = runtime.model.start();
        assert!(runtime.model.acquire(session));
        let at = Instant::now() - INPUT_TTL;
        assert!(runtime.model.input_ordered(
            session,
            1,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            at
        ));
        runtime.ticket = Some(ticket);
        assert!(runtime.watchdog_due(Instant::now()));
        assert_eq!(runtime.revoke(&connection), Some(ticket));
        assert_eq!(runtime.model.input, Axes::default());
        assert_eq!(runtime.model.state.velocity, Axes::default());
        assert!(connection.claim_replay().is_ok());
        assert!(started.elapsed() < Duration::from_millis(250));
        let _ = unblock_tx.send(());
        let _ = unblock_tx.send(());
    }
    #[test]
    fn failed_or_panicked_writer_fails_closed() {
        for panic in [false, true] {
            let log = Journal::new(2, move |_| {
                if panic {
                    panic!("test writer");
                }
                Err("write failed".into())
            });
            log.send(1, "virtual_started").unwrap();
            let until = Instant::now() + Duration::from_secs(1);
            while log.healthy() && Instant::now() < until {
                std::thread::yield_now();
            }
            assert!(!log.healthy());
            assert!(log.send(1, "authority_granted").is_err());
        }
    }
    #[test]
    fn inaccessible_journal_directory_fails() {
        let path =
            std::env::temp_dir().join(format!("firescene-journal-file-{}", std::process::id()));
        std::fs::write(&path, b"fixture").unwrap();
        assert!(append_journal_at(&path, 1, "virtual_started").is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn virtual_journal_is_distinct_jsonl_without_input_values() {
        let path = std::env::temp_dir().join(format!(
            "firescene-virtual-journal-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        append_journal_at(&path, 7, "virtual_started").unwrap();
        append_journal_at(&path, 7, "input_reset").unwrap();
        let data = std::fs::read_to_string(path.join("virtual-control-events.jsonl")).unwrap();
        let events: Vec<serde_json::Value> = data
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["source"], "VIRTUAL_MEMORY");
        assert_eq!(events[1]["code"], "input_reset");
        assert!(events.iter().all(|e| e.get("axes").is_none()));
        std::fs::remove_file(path.join("virtual-control-events.jsonl")).unwrap();
        std::fs::remove_dir(path).unwrap();
    }
    #[test]
    fn expired_lease_revokes_without_poll_and_old_ticket_cannot_release_new_one() {
        let connection = crate::connection::Manager::default();
        let mut runtime = Runtime::default();
        let ticket = connection.claim_virtual().unwrap();
        let session = runtime.model.start();
        assert!(runtime.model.acquire(session));
        let at = Instant::now();
        assert!(runtime.model.input_ordered(
            session,
            1,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            at
        ));
        runtime.ticket = Some(ticket);
        assert!(!runtime.watchdog_due(at + Duration::from_millis(499)));
        assert!(runtime.watchdog_due(at + INPUT_TTL));
        assert_eq!(runtime.revoke(&connection), Some(ticket));
        assert_eq!(runtime.model.state.velocity, Axes::default());
        assert_eq!(runtime.model.input, Axes::default());
        assert!(!runtime.model.input_ordered(
            session,
            2,
            Axes {
                forward: 1.0,
                ..Axes::default()
            },
            at + INPUT_TTL
        ));
        let newer = connection.claim_virtual().unwrap();
        assert!(!connection.release_virtual(ticket));
        assert!(connection.release_virtual(newer));
    }
}
#[derive(Default)]
pub struct VirtualManager {
    inner: Mutex<Runtime>,
    focus_lost: AtomicBool,
    journal: OnceLock<Journal>,
}
impl VirtualManager {
    pub fn mark_focus_lost(&self) {
        self.focus_lost.store(true, Ordering::SeqCst);
    }
}
struct Runtime {
    model: VirtualControl,
    ticket: Option<u64>,
    model_session: u64,
    last_step: Instant,
    safety: crate::safety::Monitor,
    simulated: crate::telemetry::View,
    simulated_connection: bool,
    simulated_fault: bool,
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            model: VirtualControl::default(),
            ticket: None,
            model_session: 0,
            last_step: Instant::now(),
            safety: crate::safety::Monitor::default(),
            simulated: crate::telemetry::View::default(),
            simulated_connection: false,
            simulated_fault: false,
        }
    }
}
#[derive(Serialize)]
pub struct VirtualView {
    ticket: Option<u64>,
    authority: String,
    north: f64,
    east: f64,
    up: f64,
    yaw_deg: f64,
    velocity: Axes,
    reason: String,
    safety: String,
}
impl Runtime {
    // Called with the runtime mutex held, including by stop and watchdog.
    // Invalidating the model precedes releasing the ticket and any disk I/O.
    fn revoke(&mut self, connection: &crate::connection::Manager) -> Option<u64> {
        let ticket = self.ticket?;
        self.model.stop();
        if !connection.release_virtual(ticket) {
            return None;
        }
        self.ticket = None;
        Some(ticket)
    }

    fn watchdog_due(&self, now: Instant) -> bool {
        self.ticket.is_some()
            && (self.model.authority() != Authority::Control
                || self.model.updated.is_none_or(|at| {
                    now.checked_duration_since(at)
                        .is_none_or(|age| age >= INPUT_TTL)
                }))
    }
    fn reset_simulated_safety(&mut self) {
        self.simulated = crate::telemetry::View::default();
        for (key, value) in [
            ("battery", 80.0),
            ("gps_fix", 3.0),
            ("satellites", 10.0),
            ("sensors_present", 3.0),
            ("sensors_enabled", 3.0),
            ("sensors_health", 3.0),
            ("roll", 0.0),
            ("pitch", 0.0),
            ("mav_state", 3.0),
        ] {
            self.simulated.fields.insert(
                key.into(),
                crate::telemetry::Field {
                    value: Some(value),
                    text: None,
                    unit: "",
                    status: "LIVE",
                    timestamp_ms: 1,
                    source: "VIRTUAL_TEST",
                    system_id: 42,
                    component_id: 1,
                },
            );
        }
        for (key, text) in [
            ("armed", "DISARMED"),
            ("mode", "Position"),
            ("landed", "ON_GROUND"),
        ] {
            self.simulated.fields.insert(
                key.into(),
                crate::telemetry::Field {
                    value: None,
                    text: Some(text.into()),
                    unit: "",
                    status: "LIVE",
                    timestamp_ms: 1,
                    source: "VIRTUAL_TEST",
                    system_id: 42,
                    component_id: 1,
                },
            );
        }
        self.simulated_connection = true;
        self.simulated_fault = false;
        self.safety.new_session("VIRTUAL_TEST");
    }
    fn safety_state(&mut self) -> String {
        self.safety
            .evaluate(
                if self.simulated_connection {
                    "CONNECTED"
                } else {
                    "CONNECTION_LOST"
                },
                "VIRTUAL_TEST",
                &self.simulated,
                if self.simulated_connection {
                    Some((42, 1))
                } else {
                    None
                },
            )
            .overall
    }
    fn view(&mut self) -> VirtualView {
        let now = Instant::now();
        let step = now.saturating_duration_since(self.last_step);
        self.last_step = now;
        let state = self.model.advance(now, step);
        let safety = self.safety_state();
        let authority = match self.model.authority() {
            Authority::Offline => "OFFLINE",
            Authority::Ready => "SIMULATION READY",
            Authority::Control => "SIMULATION CONTROL",
            Authority::Locked => "CONTROL LOCKED",
        };
        VirtualView {
            ticket: self.ticket,
            authority: authority.into(),
            north: state.north,
            east: state.east,
            up: state.up,
            yaw_deg: state.yaw_deg,
            velocity: state.velocity,
            reason: if self.model.authority() == Authority::Locked {
                "입력 만료 또는 안전 잠금 · 다시 시작 필요".into()
            } else {
                String::new()
            },
            safety,
        }
    }
}

/// Independent of the webview and its IPC polling. Fail closed when the main
/// window disappears, loses focus, is minimized, or the input lease expires.
pub fn start_watchdog(app: tauri::AppHandle) {
    use tauri::Manager as _;
    let dir = app.path().app_data_dir();
    let log = Journal::new(128, move |event| {
        let dir = dir
            .as_ref()
            .map_err(|_| "가상 이벤트 경로 오류".to_string())?;
        append_event_at(dir, event)
    });
    // Probe asynchronously. Start remains unavailable until it completes.
    let _ = log.send(0, "");
    let _ = app.state::<VirtualManager>().journal.set(log);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));
        watchdog_tick(
            &app.state::<VirtualManager>(),
            &app.state::<crate::connection::Manager>(),
        );
    });
}
fn watchdog_tick(manager: &VirtualManager, connection: &crate::connection::Manager) {
    let Ok(mut runtime) = manager.inner.lock() else {
        return;
    };
    if runtime.ticket.is_none() {
        return;
    }
    // Never synchronously query the GUI thread here: a hung webview must
    // not prevent lease expiry. Window events provide an early signal.
    let reason = if manager.journal.get().is_none_or(|j| !j.healthy()) {
        Some("journal_error")
    } else if manager.focus_lost.load(Ordering::SeqCst) {
        Some("focus_lost")
    } else if runtime.model.authority() == Authority::Locked {
        Some("safety_lock")
    } else if runtime.watchdog_due(Instant::now()) {
        Some("input_timeout")
    } else {
        None
    };
    if let Some(reason) = reason {
        let had_fault = runtime.simulated_fault;
        if let Some(ticket) = runtime.revoke(connection) {
            // No new session can start until the runtime lock is released.
            // Logging failure must never undo the already completed revoke.
            if let Some(log) = manager.journal.get() {
                let _ = log.send(ticket, reason);
                if had_fault {
                    let _ = log.send(ticket, "simulated_warning_ended");
                }
                let _ = log.send(ticket, "authority_released");
                let _ = log.send(ticket, "virtual_stopped");
            }
        }
    }
}
#[tauri::command]
pub fn virtual_start(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    connection: tauri::State<'_, crate::connection::Manager>,
    manager: tauri::State<'_, VirtualManager>,
) -> Result<VirtualView, String> {
    if window.label() != "main"
        || window.is_focused().ok() != Some(true)
        || window.is_minimized().ok() != Some(false)
    {
        return Err("활성화된 주 창에서만 가상 제어를 시작할 수 있습니다".into());
    }
    let mut runtime = manager.inner.lock().map_err(|_| "가상 제어 상태 오류")?;
    if runtime.ticket.is_some() {
        return Err("기존 가상 세션을 종료하세요".into());
    }
    let log = manager.journal.get().ok_or("가상 이벤트 기록 미준비")?;
    if !log.healthy() || log.pending.load(Ordering::SeqCst) != 0 {
        return Err("가상 이벤트 기록 오류 또는 처리 대기 · 시작 불가".into());
    }
    manager.focus_lost.store(false, Ordering::SeqCst);
    let ticket = connection.claim_virtual()?;
    let session = runtime.model.start();
    if !runtime.model.acquire(session)
        || !runtime
            .model
            .input_ordered(session, 0, Axes::default(), Instant::now())
    {
        connection.release_virtual(ticket);
        return Err("가상 세션 시작 실패".into());
    }
    runtime.ticket = Some(ticket);
    runtime.model_session = session;
    runtime.last_step = Instant::now();
    runtime.reset_simulated_safety();
    if runtime.safety_state() != "NORMAL" {
        runtime.model.stop();
        runtime.ticket = None;
        connection.release_virtual(ticket);
        return Err("가상 안전 상태 확인 실패".into());
    }
    if let Err(err) = journal(&app, ticket, "virtual_started")
        .and_then(|_| journal(&app, ticket, "authority_granted"))
    {
        runtime.model.stop();
        runtime.ticket = None;
        connection.release_virtual(ticket);
        return Err(err);
    }
    Ok(runtime.view())
}
#[tauri::command]
pub fn virtual_input(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    manager: tauri::State<'_, VirtualManager>,
    ticket: u64,
    sequence: u64,
    axes: Axes,
) -> Result<VirtualView, String> {
    // Window APIs may need the GUI thread. Never wait for it while holding the
    // runtime mutex needed by the independent watchdog.
    let window_usable = window.label() == "main"
        && window.is_focused().ok() == Some(true)
        && window.is_minimized().ok() == Some(false);
    let mut runtime = manager.inner.lock().map_err(|_| "가상 제어 상태 오류")?;
    if runtime.ticket != Some(ticket) {
        return Err("가상 세션이 일치하지 않습니다".into());
    }
    if manager.journal.get().is_none_or(|j| !j.healthy()) {
        runtime.model.lock();
        return Err("가상 이벤트 기록 오류 · 가상 입력 잠김".into());
    }
    if !window_usable {
        runtime.model.lock();
        journal(&app, ticket, "focus_lost_lock")?;
        return Err("주 창 포커스 상실 또는 최소화 · 가상 입력 잠김".into());
    }
    if runtime.safety_state() != "NORMAL" {
        runtime.model.lock();
        journal(&app, ticket, "safety_lock")?;
        return Err("모의 안전 상태로 가상 제어 잠김".into());
    }
    let now = Instant::now();
    let step = now.saturating_duration_since(runtime.last_step);
    runtime.last_step = now;
    let previous = runtime.model.input;
    runtime.model.advance(now, step);
    if runtime.model.authority() == Authority::Locked {
        journal(&app, ticket, "input_timeout")?;
        return Err("입력 시간 초과 · 가상 제어 잠김".into());
    }
    let id = runtime.model_session;
    if !runtime.model.input_ordered(id, sequence, axes, now) {
        if runtime.model.authority() == Authority::Locked {
            journal(&app, ticket, "safety_lock")?;
        }
        return Err("입력 거부 또는 권한 잠김".into());
    }
    if previous != Axes::default() && axes == Axes::default() {
        if let Err(err) = journal(&app, ticket, "input_reset") {
            runtime.model.lock();
            return Err(err);
        }
    }
    Ok(runtime.view())
}
#[tauri::command]
pub fn virtual_status(
    app: tauri::AppHandle,
    manager: tauri::State<'_, VirtualManager>,
) -> Result<VirtualView, String> {
    let mut runtime = manager.inner.lock().map_err(|_| "가상 제어 상태 오류")?;
    let previous = runtime.model.authority();
    let view = runtime.view();
    if previous == Authority::Control && runtime.model.authority() == Authority::Locked {
        if let Some(ticket) = runtime.ticket {
            journal(&app, ticket, "input_timeout")?;
        }
    }
    Ok(view)
}
#[tauri::command]
pub fn virtual_stop(
    app: tauri::AppHandle,
    connection: tauri::State<'_, crate::connection::Manager>,
    manager: tauri::State<'_, VirtualManager>,
    ticket: u64,
) -> Result<(), String> {
    let mut runtime = manager.inner.lock().map_err(|_| "가상 제어 상태 오류")?;
    if runtime.ticket != Some(ticket) {
        return Err("가상 세션이 일치하지 않습니다".into());
    }
    let had_fault = runtime.simulated_fault;
    if runtime.revoke(&connection).is_none() {
        return Err("가상 권한 해제 오류".into());
    }
    if had_fault {
        journal(&app, ticket, "simulated_warning_ended")?;
    }
    journal(&app, ticket, "authority_released")
        .and_then(|_| journal(&app, ticket, "virtual_stopped"))
}
#[tauri::command]
pub fn virtual_lock(
    app: tauri::AppHandle,
    manager: tauri::State<'_, VirtualManager>,
    ticket: u64,
) -> Result<VirtualView, String> {
    let mut runtime = manager.inner.lock().map_err(|_| "가상 제어 상태 오류")?;
    if runtime.ticket != Some(ticket) {
        return Err("가상 세션이 일치하지 않습니다".into());
    }
    runtime.model.lock();
    journal(&app, ticket, "safety_lock")?;
    Ok(runtime.view())
}
#[tauri::command]
pub fn virtual_fault(
    app: tauri::AppHandle,
    manager: tauri::State<'_, VirtualManager>,
    ticket: u64,
    fault: String,
) -> Result<VirtualView, String> {
    let mut runtime = manager.inner.lock().map_err(|_| "가상 제어 상태 오류")?;
    // Explicitly enabled only in isolated development QA: restore simulated
    // observations after authority has already been fully revoked.
    if fault == "qa_recover"
        && std::env::var_os("FIRESCENE_QA_RECOVERY").as_deref() == Some(std::ffi::OsStr::new("1"))
    {
        if runtime.ticket.is_some() || runtime.model.authority() != Authority::Offline {
            return Err("가상 세션 종료 후에만 시험 상태를 복구할 수 있습니다".into());
        }
        runtime.reset_simulated_safety();
        return Ok(runtime.view());
    }
    if runtime.ticket != Some(ticket) {
        return Err("가상 세션이 일치하지 않습니다".into());
    }
    match fault.as_str() {
        "connection_loss" => runtime.simulated_connection = false,
        "battery_low" => runtime.simulated.fields.get_mut("battery").unwrap().value = Some(10.0),
        "gps_loss" => runtime.simulated.fields.get_mut("gps_fix").unwrap().status = "STALE",
        "attitude_loss" => runtime.simulated.fields.get_mut("roll").unwrap().status = "STALE",
        "sensor_stale" => {
            runtime
                .simulated
                .fields
                .get_mut("sensors_health")
                .unwrap()
                .status = "STALE"
        }
        _ => return Err("지원하지 않는 모의 상황".into()),
    }
    let state = runtime.safety_state();
    runtime.simulated_fault = true;
    if state == "NORMAL" {
        runtime.model.lock();
        return Err("모의 위험 감지 실패 · 가상 제어 잠김".into());
    }
    runtime.model.lock();
    manager
        .journal
        .get()
        .ok_or("가상 이벤트 기록 미준비")?
        .send_reason(ticket, "simulated_warning", Some(&fault))?;
    journal(&app, ticket, "safety_lock")?;
    Ok(runtime.view())
}
#[tauri::command]
pub fn virtual_replay_start(
    connection: tauri::State<'_, crate::connection::Manager>,
) -> Result<u64, String> {
    connection.claim_replay()
}
#[tauri::command]
pub fn virtual_replay_stop(
    connection: tauri::State<'_, crate::connection::Manager>,
    ticket: u64,
) -> Result<(), String> {
    if connection.release_replay(ticket) {
        Ok(())
    } else {
        Err("재생 세션이 일치하지 않습니다".into())
    }
}
