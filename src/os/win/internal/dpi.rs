use winapi::shared::minwindef::{BOOL, TRUE};
use winapi::shared::ntdef::HRESULT;
use winapi::shared::winerror::S_OK;

use crate::os::win::Library;


macro_rules! activate {
    ($module: expr, $function: ty[$param: expr] == $ok: expr) => {
        unsafe {
            let function_name = concat![stringify!($function), '\0'];
            let function_name = function_name.as_bytes();

            if let Ok(library) = Library::open($module) {
                if let Ok(function) = library.find::<$function>(function_name) {
                    if function($param) == $ok {
                        return;
                    }
                }
            }
        }
    };
}


pub(crate) fn activate_dpi_awareness() {
    type SetProcessDpiAwareness = extern "system" fn(value: u32) -> HRESULT;
    type SetProcessDpiAwarenessContext = extern "system" fn(value: usize) -> BOOL;

    activate!("user32.dll", SetProcessDpiAwarenessContext[-4isize as usize] == TRUE); // per-monitor-v2 | windows 10 1703+
    activate!("shcore.dll", SetProcessDpiAwareness[2] == S_OK); // per-monitor-v1 | windows 8.1
    activate!("shcore.dll", SetProcessDpiAwareness[1] == S_OK); // system-aware | windows 8.1
}
