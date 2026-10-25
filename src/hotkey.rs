use iced::{
    Subscription,
    futures::{SinkExt, StreamExt},
};

#[derive(Debug, Clone)]
pub enum Event {
    Ready,
    Toggle,
    Capture,
    CaptureReady,
    CaptureFailed(String),
    Failed(String),
}

pub fn subscription() -> Subscription<Event> {
    Subscription::run(events)
}
fn events() -> impl iced::futures::Stream<Item = Event> {
    iced::stream::channel(8, async |mut output| {
        let (sender, mut receiver) = iced::futures::channel::mpsc::unbounded();
        #[cfg(windows)]
        std::thread::spawn(move || {
            use global_hotkey::{
                GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
                hotkey::{Code, HotKey, Modifiers},
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, MSG, TranslateMessage,
            };
            let manager = match GlobalHotKeyManager::new() {
                Ok(m) => m,
                Err(e) => {
                    let _ = sender.unbounded_send(Event::Failed(e.to_string()));
                    return;
                }
            };
            let key = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
            if let Err(e) = manager.register(key) {
                let _ = sender.unbounded_send(Event::Failed(e.to_string()));
                return;
            }
            let _ = sender.unbounded_send(Event::Ready);
            let capture = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyR);
            let capture_registered = match manager.register(capture) {
                Ok(()) => {
                    let _ = sender.unbounded_send(Event::CaptureReady);
                    true
                }
                Err(e) => {
                    let _ = sender.unbounded_send(Event::CaptureFailed(e.to_string()));
                    false
                }
            };
            let callback_sender = sender.clone();
            GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
                if event.id == key.id() && event.state == HotKeyState::Pressed {
                    let _ = callback_sender.unbounded_send(Event::Toggle);
                }
                if capture_registered
                    && event.id == capture.id()
                    && event.state == HotKeyState::Pressed
                {
                    let _ = callback_sender.unbounded_send(Event::Capture);
                }
            }));
            // This thread owns the registration and its Win32 message loop.
            // No game handles, hooks, memory access or injected code are used.
            unsafe {
                let mut message: MSG = std::mem::zeroed();
                loop {
                    let result = GetMessageW(&mut message, std::ptr::null_mut(), 0, 0);
                    if result <= 0 {
                        break;
                    }
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            let _ = manager.unregister(key);
            let _ = sender.unbounded_send(Event::Failed(
                "Shortcut unavailable: the message service stopped.".into(),
            ));
        });
        #[cfg(not(windows))]
        {
            let _ = sender.unbounded_send(Event::Failed(
                "Global shortcut is available on Windows only.".into(),
            ));
        }
        while let Some(event) = receiver.next().await {
            let _ = output.send(event).await;
        }
        iced::futures::future::pending::<()>().await;
    })
}
