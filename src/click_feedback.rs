//! Cursor feedback driven by completed clicks, independently of input timing.
use crate::click_assist::Activity;
use std::time::Duration;

const PULSE_TIME: Duration = Duration::from_millis(240);

#[derive(Default)]
struct Pulse {
    clicks: u64,
    started: Option<Duration>,
}
impl Pulse {
    fn update(&mut self, activity: Activity, now: Duration) -> Option<f32> {
        if !activity.active || activity.clicks == 0 {
            *self = Self::default();
            return None;
        }
        if activity.clicks < self.clicks {
            self.started = None;
        }
        let fresh_click = activity.clicks != self.clicks;
        self.clicks = activity.clicks;
        if fresh_click
            && self
                .started
                .is_none_or(|start| now.saturating_sub(start) >= PULSE_TIME)
        {
            self.started = Some(now);
        }
        let elapsed = now.saturating_sub(self.started?);
        (elapsed < PULSE_TIME).then(|| elapsed.as_secs_f32() / PULSE_TIME.as_secs_f32())
    }
}

fn paint(pixels: &mut [u32], side: usize, scale: f32, phase: f32) {
    let radius = 10. + 16. * (1. - (1. - phase).powi(3));
    let center = (side as f32 - 1.) / 2.;
    for (index, pixel) in pixels.iter_mut().enumerate() {
        let x = (index % side) as f32 - center;
        let y = (index / side) as f32 - center;
        let edge = ((x.hypot(y) / scale) - radius).abs();
        let ring = (1. - edge / 1.2).max(0.) * 0.68;
        let glow = (-0.5 * (edge / 3.).powi(2)).exp() * 0.14;
        let alpha = ((ring + glow) * (1. - phase) * 255.).round() as u32;
        // UpdateLayeredWindow requires premultiplied BGRA, including the glow.
        *pixel = (alpha << 24) | (alpha << 16) | ((110 * alpha / 255) << 8) | (48 * alpha / 255);
    }
}

#[cfg(windows)]
pub(crate) use windows::run;

#[cfg(windows)]
pub(crate) fn preview() {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    let activity = Arc::new(Mutex::new(Activity {
        active: true,
        clicks: 0,
    }));
    let shutdown = Arc::new(AtomicBool::new(false));
    let (source, done) = (activity.clone(), shutdown.clone());
    let producer = std::thread::spawn(move || {
        for clicks in 1..=180 {
            source.lock().unwrap().clicks = clicks;
            std::thread::sleep(Duration::from_millis(16));
        }
        done.store(true, Ordering::SeqCst);
    });
    run(activity, shutdown);
    let _ = producer.join();
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        ptr,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        time::Instant,
    };
    use windows_sys::Win32::{
        Foundation::{HWND, POINT, SIZE},
        Graphics::Gdi::{
            AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
            CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, HBITMAP,
            HDC, HGDIOBJ, SelectObject,
        },
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow,
                SetThreadDpiAwarenessContext,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, GetCursorPos,
                GetForegroundWindow, MSG, PM_REMOVE, PeekMessageW, SW_HIDE, SW_SHOWNOACTIVATE,
                ShowWindow, TranslateMessage, ULW_ALPHA, UpdateLayeredWindow, WS_EX_LAYERED,
                WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
            },
        },
    };

    struct Surface {
        window: HWND,
        dc: HDC,
        bitmap: HBITMAP,
        previous: HGDIOBJ,
        bits: *mut u32,
        side: i32,
        scale: f32,
    }
    impl Surface {
        fn new(scale: f32) -> Option<Self> {
            let side = (72. * scale).ceil() as i32;
            let mut surface = Self {
                window: ptr::null_mut(),
                dc: ptr::null_mut(),
                bitmap: ptr::null_mut(),
                previous: ptr::null_mut(),
                bits: ptr::null_mut(),
                side,
                scale,
            };
            unsafe {
                surface.window = CreateWindowExW(
                    WS_EX_LAYERED
                        | WS_EX_TRANSPARENT
                        | WS_EX_NOACTIVATE
                        | WS_EX_TOOLWINDOW
                        | WS_EX_TOPMOST,
                    windows_sys::w!("STATIC"),
                    windows_sys::w!("WARDOGS Click feedback"),
                    WS_POPUP,
                    0,
                    0,
                    side,
                    side,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null(),
                );
                if surface.window.is_null() {
                    return None;
                }
                surface.dc = CreateCompatibleDC(ptr::null_mut());
                if surface.dc.is_null() {
                    return None;
                }
                let info = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: side,
                        biHeight: -side,
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB,
                        ..std::mem::zeroed()
                    },
                    ..std::mem::zeroed()
                };
                let mut bits = ptr::null_mut();
                surface.bitmap = CreateDIBSection(
                    surface.dc,
                    &info,
                    DIB_RGB_COLORS,
                    &mut bits,
                    ptr::null_mut(),
                    0,
                );
                if surface.bitmap.is_null() || bits.is_null() {
                    return None;
                }
                surface.bits = bits.cast();
                surface.previous = SelectObject(surface.dc, surface.bitmap);
                if surface.previous.is_null() || surface.previous as isize == -1 {
                    surface.previous = ptr::null_mut();
                    return None;
                }
            }
            Some(surface)
        }
        fn show(&mut self, cursor: POINT, phase: f32) -> bool {
            unsafe {
                windows_sys::Win32::Graphics::Gdi::GdiFlush();
                let pixels =
                    std::slice::from_raw_parts_mut(self.bits, (self.side * self.side) as usize);
                paint(pixels, self.side as usize, self.scale, phase);
                let destination = POINT {
                    x: cursor.x - self.side / 2,
                    y: cursor.y - self.side / 2,
                };
                let size = SIZE {
                    cx: self.side,
                    cy: self.side,
                };
                let source = POINT { x: 0, y: 0 };
                let blend = BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: AC_SRC_ALPHA as u8,
                };
                if UpdateLayeredWindow(
                    self.window,
                    ptr::null_mut(),
                    &destination,
                    &size,
                    self.dc,
                    &source,
                    0,
                    &blend,
                    ULW_ALPHA,
                ) == 0
                {
                    return false;
                }
                ShowWindow(self.window, SW_SHOWNOACTIVATE);
            }
            true
        }
        fn hide(&self) {
            unsafe {
                ShowWindow(self.window, SW_HIDE);
            }
        }
    }
    impl Drop for Surface {
        fn drop(&mut self) {
            unsafe {
                if !self.previous.is_null() {
                    SelectObject(self.dc, self.previous);
                }
                if !self.bitmap.is_null() {
                    DeleteObject(self.bitmap);
                }
                if !self.dc.is_null() {
                    DeleteDC(self.dc);
                }
                if !self.window.is_null() {
                    DestroyWindow(self.window);
                }
            }
        }
    }

    pub(crate) fn run(activity: Arc<Mutex<Activity>>, shutdown: Arc<AtomicBool>) {
        unsafe {
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let epoch = Instant::now();
        let mut pulse = Pulse::default();
        let mut surface: Option<Surface> = None;
        while !shutdown.load(Ordering::SeqCst) {
            let mut message: MSG = unsafe { std::mem::zeroed() };
            unsafe {
                while PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            let state = *activity.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(phase) = pulse.update(state, epoch.elapsed()) {
                let scale = unsafe { GetDpiForWindow(GetForegroundWindow()) }.max(96) as f32 / 96.;
                if surface.as_ref().is_none_or(|s| s.scale != scale) {
                    surface = Surface::new(scale);
                }
                let Some(surface) = surface.as_mut() else {
                    return;
                };
                let mut cursor = POINT { x: 0, y: 0 };
                if unsafe { GetCursorPos(&mut cursor) } == 0 || !surface.show(cursor, phase) {
                    surface.hide();
                }
            } else if let Some(surface) = &surface {
                surface.hide();
            }
            std::thread::sleep(Duration::from_millis(if state.active { 16 } else { 50 }));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use windows_sys::Win32::{
            Graphics::Gdi::GdiFlush,
            System::Threading::{GR_GDIOBJECTS, GetCurrentProcess, GetGuiResources},
            UI::WindowsAndMessaging::{GWL_EXSTYLE, GetWindowLongPtrW, IsWindow, IsWindowVisible},
        };

        #[test]
        #[ignore = "native desktop visual check; sends no mouse input"]
        fn native_surface_keeps_focus_passes_input_and_releases_resources() {
            unsafe {
                let foreground = GetForegroundWindow();
                let mut cursor = POINT { x: 0, y: 0 };
                assert_ne!(GetCursorPos(&mut cursor), 0);
                let mut surface = Surface::new(1.).expect("Create cursor feedback surface");
                let window = surface.window;
                let required =
                    WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
                assert_eq!(
                    GetWindowLongPtrW(window, GWL_EXSTYLE) as u32 & required,
                    required
                );
                assert!(surface.show(cursor, 0.2));
                assert_eq!(GetForegroundWindow(), foreground);
                assert_ne!(IsWindowVisible(window), 0);
                surface.hide();
                assert_eq!(IsWindowVisible(window), 0);
                drop(surface);
                assert_eq!(IsWindow(window), 0);
                GdiFlush();
                let resources = GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS);
                for _ in 0..10 {
                    let mut repeated = Surface::new(1.).unwrap();
                    assert!(repeated.show(cursor, 0.2));
                    repeated.hide();
                    drop(repeated);
                }
                GdiFlush();
                assert_eq!(
                    GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS),
                    resources
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pulses_require_completed_clicks_and_coalesce_fast_input() {
        let mut pulse = Pulse::default();
        let mut activity = Activity {
            active: true,
            clicks: 0,
        };
        assert_eq!(pulse.update(activity, Duration::ZERO), None);
        activity.clicks = 1;
        assert_eq!(pulse.update(activity, Duration::ZERO), Some(0.));
        activity.clicks = 50;
        assert_eq!(
            pulse.update(activity, Duration::from_millis(120)),
            Some(0.5)
        );
        activity.clicks = 100;
        assert_eq!(pulse.update(activity, PULSE_TIME), Some(0.));
        assert_eq!(pulse.update(activity, PULSE_TIME * 2), None);
        activity.active = false;
        assert_eq!(pulse.update(activity, PULSE_TIME * 2), None);
        activity.active = true;
        activity.clicks = 1;
        assert_eq!(pulse.update(activity, PULSE_TIME * 3), Some(0.));
    }
    #[test]
    fn ring_has_a_transparent_center_and_premultiplied_fading_edges() {
        for scale in [1., 1.5, 2.] {
            let side = (72. * scale) as usize;
            let mut pixels = vec![0; side * side];
            paint(&mut pixels, side, scale, 0.);
            assert_eq!(pixels[side / 2 * side + side / 2], 0);
            assert_eq!(pixels[0], 0);
            assert!(pixels.iter().any(|p| p >> 24 > 100));
            assert!(pixels.iter().all(|p| {
                let alpha = p >> 24;
                ((p >> 16) & 255) <= alpha && ((p >> 8) & 255) <= alpha && (p & 255) <= alpha
            }));
            paint(&mut pixels, side, scale, 1.);
            assert!(pixels.iter().all(|p| *p == 0));
        }
    }
}
