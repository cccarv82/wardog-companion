//! Explicit, bounded desktop mouse input. No hooks, injection or game memory access.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

pub struct Assist {
    activity: Arc<Mutex<Activity>>,
    enabled: Arc<AtomicBool>,
    interval: Arc<AtomicU32>,
    status: Arc<Mutex<String>>,
    shutdown: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct Activity {
    pub active: bool,
    pub clicks: u64,
}
impl Activity {
    fn begin(&mut self) {
        self.active = true;
        self.clicks = 0;
    }
    fn released(&mut self) {
        self.clicks = self.clicks.saturating_add(1);
    }
    pub fn label(self, enabled: bool) -> String {
        let state = if !enabled {
            "Off"
        } else if self.active {
            "Clicking"
        } else {
            "Ready"
        };
        if self.clicks > 0 || self.active {
            format!("Auto click: {state} · {} clicks sent", self.clicks)
        } else {
            format!("Auto click: {state}")
        }
    }
}
impl Assist {
    pub fn new() -> Self {
        let activity = Arc::new(Mutex::new(Activity::default()));
        let enabled = Arc::new(AtomicBool::new(false));
        let interval = Arc::new(AtomicU32::new(2));
        let status = Arc::new(Mutex::new("Off · enable for this session".into()));
        let shutdown = Arc::new(AtomicBool::new(false));
        #[cfg(windows)]
        let worker = {
            let (e, i, s, q, a) = (
                enabled.clone(),
                interval.clone(),
                status.clone(),
                shutdown.clone(),
                activity.clone(),
            );
            std::thread::Builder::new()
                .name("click-assist".into())
                .spawn(move || windows::run(e, i, s, q, a))
                .ok()
        };
        #[cfg(not(windows))]
        let worker = None;
        if worker.is_none() {
            *status.lock().unwrap() = "Unavailable on this system".into();
        }
        Self {
            activity,
            enabled,
            interval,
            status,
            shutdown,
            worker,
        }
    }
    pub fn enable(&self, enabled: bool) {
        self.enabled
            .store(enabled && self.worker.is_some(), Ordering::SeqCst);
        if !self.enabled() {
            self.activity
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .active = false;
        }
        *self.status.lock().unwrap_or_else(|e| e.into_inner()) = if self.enabled() {
            "Ready · point at the team button and press F8"
        } else {
            "Off · enable for this session"
        }
        .into();
    }
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
    pub fn activity(&self) -> Activity {
        *self.activity.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn interval(&self) -> u32 {
        self.interval.load(Ordering::Relaxed)
    }
    pub fn set_interval(&self, ms: u32) {
        self.interval.store(ms.clamp(2, 100), Ordering::Relaxed);
    }
    pub fn status(&self) -> String {
        self.status
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}
impl Drop for Assist {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Default)]
struct Gate {
    previous: bool,
    active: bool,
    armed: bool,
}

#[derive(Default)]
struct Cycle {
    down: bool,
    last_press: Option<std::time::Duration>,
}
impl Cycle {
    fn step(&mut self, active: bool, now: std::time::Duration, interval: u32) -> Option<bool> {
        let elapsed = self.last_press.map(|last| now.saturating_sub(last));
        if self.down
            && (!active || elapsed.is_some_and(|t| t >= std::time::Duration::from_millis(1)))
        {
            self.down = false;
            Some(false)
        } else if active
            && !self.down
            && elapsed.is_none_or(|t| {
                t >= std::time::Duration::from_millis(interval.clamp(2, 100) as u64)
            })
        {
            self.down = true;
            self.last_press = Some(now);
            Some(true)
        } else {
            None
        }
    }
}
impl Gate {
    fn step(&mut self, enabled: bool, key: bool, escape: bool, target: bool, toggle: bool) -> bool {
        let pressed = key && !self.previous;
        self.previous = key;
        if !enabled || escape || !target {
            self.active = false;
            self.armed = false;
            return false;
        }
        if !key {
            self.armed = true;
        }
        if pressed && self.armed {
            self.active = if toggle { !self.active } else { true };
        }
        if !toggle && !key {
            self.active = false;
        }
        self.active
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        ptr,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, POINT},
        System::Threading::{
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, OpenProcess,
            PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, SetWaitableTimer,
            TIMER_ALL_ACCESS, WaitForSingleObject,
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN,
                MOUSEEVENTF_LEFTUP, MOUSEINPUT, SendInput, VK_ESCAPE, VK_F8, VK_LBUTTON,
            },
            WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId},
        },
    };
    struct Timer(HANDLE);
    impl Timer {
        fn wait(&self) {
            if self.0.is_null() {
                std::thread::sleep(Duration::from_millis(1));
                return;
            }
            unsafe {
                let due = -10_000i64; // Relative 1 ms; never busy-spin.
                if SetWaitableTimer(self.0, &due, 0, None, ptr::null(), 0) != 0 {
                    WaitForSingleObject(self.0, 20);
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
    }
    impl Drop for Timer {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }
    fn target() -> Option<isize> {
        unsafe {
            let window = GetForegroundWindow();
            if window.is_null() {
                return None;
            }
            let mut pid = 0;
            GetWindowThreadProcessId(window, &mut pid);
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() {
                return None;
            }
            let mut path = [0u16; 1024];
            let mut len = path.len() as u32;
            let ok = QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut len);
            CloseHandle(process);
            if ok == 0 {
                return None;
            }
            let path = String::from_utf16_lossy(&path[..len as usize]);
            path.rsplit('\\')
                .next()
                .is_some_and(|n| n.eq_ignore_ascii_case("WardogsClient-Win64-Shipping.exe"))
                .then_some(window as isize)
        }
    }
    fn button(down: bool) -> bool {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: if down {
                        MOUSEEVENTF_LEFTDOWN
                    } else {
                        MOUSEEVENTF_LEFTUP
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) == 1 }
    }
    pub fn run(
        enabled: Arc<AtomicBool>,
        interval: Arc<AtomicU32>,
        status: Arc<Mutex<String>>,
        shutdown: Arc<AtomicBool>,
        activity: Arc<Mutex<Activity>>,
    ) {
        let timer = Timer(unsafe {
            CreateWaitableTimerExW(
                ptr::null(),
                ptr::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS,
            )
        });
        let epoch = Instant::now();
        let mut gate = Gate::default();
        let mut cycle = Cycle::default();
        let mut origin = POINT { x: 0, y: 0 };
        let mut window = None;
        let mut was_active = false;
        let mut input_error = false;
        let mut was_enabled = false;
        let set_status = |text: &str| {
            *status.lock().unwrap_or_else(|e| e.into_inner()) = text.into();
        };
        while !shutdown.load(Ordering::SeqCst) {
            let is_enabled = enabled.load(Ordering::SeqCst);
            if is_enabled && !was_enabled {
                input_error = false;
            }
            let key = unsafe { GetAsyncKeyState(VK_F8 as i32) < 0 };
            let escape = unsafe { GetAsyncKeyState(VK_ESCAPE as i32) < 0 };
            let foreground = if is_enabled { target() } else { None };
            let mut cursor = POINT { x: 0, y: 0 };
            let cursor_ok = unsafe { GetCursorPos(&mut cursor) != 0 };
            let stationary = !was_active
                || (cursor_ok
                    && (cursor.x as i64 - origin.x as i64).abs() <= 4
                    && (cursor.y as i64 - origin.y as i64).abs() <= 4
                    && foreground == window);
            let active = gate.step(
                is_enabled,
                key,
                escape,
                foreground.is_some()
                    && cursor_ok
                    && stationary
                    && (was_active || unsafe { GetAsyncKeyState(VK_LBUTTON as i32) >= 0 }),
                true,
            );
            if active && !was_active {
                activity.lock().unwrap_or_else(|e| e.into_inner()).begin();
                origin = cursor;
                window = foreground;
                cycle.last_press = None;
                set_status("Clicking · F8 / Esc to stop");
            }
            if let Some(press) =
                cycle.step(active, epoch.elapsed(), interval.load(Ordering::Relaxed))
            {
                if press && target() != window {
                    cycle.down = false;
                    enabled.store(false, Ordering::SeqCst);
                    input_error = true;
                    set_status("Focus changed · stopped");
                } else if !button(press) {
                    // A partial/failed press is also followed by a release attempt.
                    let released = button(false);
                    if !press && released {
                        activity
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .released();
                    }
                    cycle.down = false;
                    enabled.store(false, Ordering::SeqCst);
                    input_error = true;
                    set_status("Input was rejected · stopped");
                } else if !press {
                    // Count completed, successfully sent down/up pairs, never timer ticks.
                    activity
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .released();
                }
            }
            if !active || input_error {
                activity.lock().unwrap_or_else(|e| e.into_inner()).active = false;
            }
            if !active && was_active && !input_error {
                set_status(if is_enabled {
                    "Ready · release F8, then try again"
                } else {
                    "Off · enable for this session"
                });
            }
            was_active = active;
            was_enabled = is_enabled;
            if is_enabled && (active || cycle.down) {
                timer.wait();
            } else {
                std::thread::sleep(Duration::from_millis(if is_enabled { 5 } else { 50 }));
            }
        }
        if cycle.down && !button(false) {
            let _ = button(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn displayed_count_tracks_completed_pairs_and_retains_last_attempt_until_restart() {
        use std::time::Duration;
        let mut cycle = Cycle::default();
        let mut activity = Activity::default();
        assert_eq!(activity.label(false), "Auto click: Off");
        activity.begin();
        // Five timer observations produce only two completed clicks.
        for us in [0, 500, 1000, 2000, 3000] {
            if cycle.step(true, Duration::from_micros(us), 2) == Some(false) {
                activity.released();
            }
        }
        assert_eq!(activity.clicks, 2);
        assert_eq!(activity.label(true), "Auto click: Clicking · 2 clicks sent");
        activity.active = false;
        assert_eq!(activity.label(true), "Auto click: Ready · 2 clicks sent");
        assert_eq!(activity.label(false), "Auto click: Off · 2 clicks sent");
        activity.begin();
        assert_eq!(activity.clicks, 0);
        assert!(activity.active);
    }
    #[test]
    fn cycle_has_a_real_release_gap_and_stopping_releases_pending_press() {
        use std::time::Duration;
        let mut c = Cycle::default();
        assert_eq!(c.step(false, Duration::ZERO, 2), None);
        assert_eq!(c.step(true, Duration::ZERO, 1), Some(true));
        assert_eq!(c.step(true, Duration::from_micros(999), 1), None);
        assert_eq!(c.step(true, Duration::from_millis(1), 1), Some(false));
        assert_eq!(c.step(true, Duration::from_micros(1999), 1), None);
        assert_eq!(c.step(true, Duration::from_millis(2), 1), Some(true));
        assert_eq!(c.step(false, Duration::from_micros(2001), 1), Some(false));
        assert_eq!(c.step(false, Duration::from_millis(3), 1), None);
    }
    #[test]
    fn hold_requires_fresh_press_and_stops_on_release_focus_or_escape() {
        let mut g = Gate::default();
        assert!(!g.step(true, true, false, true, false));
        assert!(!g.step(true, false, false, true, false));
        assert!(g.step(true, true, false, true, false));
        assert!(!g.step(true, false, false, true, false));
        assert!(g.step(true, true, false, true, false));
        assert!(!g.step(true, true, false, false, false));
        assert!(!g.step(true, true, false, true, false));
        g.step(true, false, false, true, false);
        assert!(g.step(true, true, false, true, false));
        assert!(!g.step(true, true, true, true, false));
        g.step(true, false, false, true, false);
        assert!(g.step(true, true, false, true, false));
        assert!(g.step(true, true, false, true, false));
        assert!(g.step(true, true, false, true, false));
    }
    #[test]
    fn continuous_attempt_survives_long_elapsed_time_until_explicit_stop() {
        use std::time::Duration;
        let mut g = Gate::default();
        let mut cycle = Cycle::default();
        g.step(true, false, false, true, true);
        assert!(g.step(true, true, false, true, true));
        for ms in [0, 2, 10_001, 60_000, 3_600_000] {
            let active = g.step(true, false, false, true, true);
            assert!(active);
            assert_eq!(cycle.step(active, Duration::from_millis(ms), 2), Some(true));
            assert_eq!(
                cycle.step(active, Duration::from_millis(ms + 1), 2),
                Some(false)
            );
        }
        assert!(!g.step(true, true, false, true, true));
        assert_eq!(cycle.step(false, Duration::from_secs(7200), 2), None);
    }
    #[test]
    fn toggle_is_edge_triggered_and_disarming_always_stops() {
        let mut g = Gate::default();
        g.step(true, false, false, true, true);
        assert!(g.step(true, true, false, true, true));
        assert!(g.step(true, true, false, true, true));
        assert!(g.step(true, false, false, true, true));
        assert!(!g.step(true, true, false, true, true));
        g.step(true, false, false, true, true);
        assert!(g.step(true, true, false, true, true));
        assert!(!g.step(false, true, false, true, true));
        assert!(!g.step(true, true, false, true, true));
    }
}
