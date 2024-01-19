use winapi::shared::minwindef::{DWORD, TRUE};
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::STD_OUTPUT_HANDLE;
use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

pub(crate) fn enable_vt_mode() -> bool {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);

        if handle != INVALID_HANDLE_VALUE {
            let mut mode: DWORD = 0;

            return GetConsoleMode(handle, &mut mode) == TRUE
                && SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == TRUE;
        }

        false
    }
}
