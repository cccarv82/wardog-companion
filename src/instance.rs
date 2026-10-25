//! One interactive companion per Windows session; also protects uninstallation.
pub struct Instance {
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HANDLE,
}

pub fn acquire() -> Result<Instance, String> {
    #[cfg(windows)]
    return acquire_named("Local\\WARDOGCompanion.71A60EC5-9053-4606-B796-2D23E8B69089");
    #[cfg(not(windows))]
    Ok(Instance {})
}

#[cfg(windows)]
fn acquire_named(name: &str) -> Result<Instance, String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError},
        System::Threading::CreateMutexW,
    };
    let wide: Vec<_> = name.encode_utf16().chain(Some(0)).collect();
    // The returned handle owns one reference to a named session-local mutex.
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
    if handle.is_null() {
        return Err("Could not initialize the application instance lock.".into());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            CloseHandle(handle);
        }
        return Err("WARDOG Companion is already running.\n\nPress Ctrl + Shift + Space to bring it back, or close the existing copy before opening another.".into());
    }
    Ok(Instance { handle })
}

#[cfg(windows)]
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

pub fn notify(message: &str) {
    #[cfg(windows)]
    {
        let text: Vec<_> = message.encode_utf16().chain(Some(0)).collect();
        let title: Vec<_> = "WARDOG Companion".encode_utf16().chain(Some(0)).collect();
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                title.as_ptr(),
                windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION,
            );
        }
    }
    #[cfg(not(windows))]
    eprintln!("{message}");
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn instance_lock_rejects_duplicates_and_releases_on_exit() {
        let name = format!("Local\\WardogCompanion-test-{}", std::process::id());
        let first = acquire_named(&name).unwrap_or_else(|e| panic!("{e}"));
        assert!(acquire_named(&name).is_err());
        drop(first);
        assert!(acquire_named(&name).is_ok());
    }
}
