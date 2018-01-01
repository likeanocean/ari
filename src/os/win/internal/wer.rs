use winapi::um::errhandlingapi::SetErrorMode;
use winapi::um::winbase::{SEM_FAILCRITICALERRORS, SEM_NOGPFAULTERRORBOX, SEM_NOOPENFILEERRORBOX};


crate fn disable_windows_error_reporting() {
    unsafe {
        if !cfg!(debug_assertions) {
            SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX);
        }
    }
}
