use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::HANDLE;


#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowsHandle {
    handle: HANDLE,
}

impl WindowsHandle {
    pub fn new(handle: HANDLE) -> WindowsHandle {
        WindowsHandle { handle: handle }
    }

    pub fn create(
        create: impl FnOnce() -> HANDLE,
        valid: impl FnOnce(HANDLE) -> bool,
    ) -> Result<WindowsHandle, std::io::Error> {
        let handle = create();

        match valid(handle) {
            true => Ok(WindowsHandle { handle }),
            false => Err(std::io::Error::last_os_error()),
        }
    }

    pub fn as_raw(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for WindowsHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                CloseHandle(self.handle);
            }
        }
    }
}
