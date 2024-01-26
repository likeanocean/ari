use winapi::shared::ntdef::HRESULT;
use winapi::shared::winerror::SUCCEEDED;

use crate::os::win::ComPtr;

/// calls a `function(..., *mut out_value) -> code`, transforming its return value into a `Result<T, std::io::Error>`.
///
/// # remarks.
///
/// this function provides you a pointer to a zero-initialized `T` which will be returned on success. the provided
/// callback is expected to fill out this value when successful.
///
/// this pattern is very common in the windows api - especially for functions returning `HRESULT`s.
///
/// # return value.
///
/// if `function(..., *mut T)` was:
///     - success: returns T
///     - failure: returns std::io::Error (GetLastError)
pub fn call<TFunction, TStatusCode, T>(function: TFunction) -> Result<T, std::io::Error>
where
    TFunction: FnOnce(*mut T) -> TStatusCode,
    TStatusCode: HrLikeStatusCode,
{
    raw_call(function).map_err(TStatusCode::error)
}

/// calls a `function(..., *mut out_value) -> code`, transforming its return value into a `Result<T, TStatusCode>`.
///
/// # remarks.
///
/// this function provides you a pointer to a zero-initialized `T` which will be returned on success. the provided
/// callback is expected to fill out this value when successful.
///
/// this pattern is very common in the windows api - especially for functions returning `HRESULT`s.
///
/// # return value.
///
/// if `function(..., *mut T)` was:
///     - success: returns T
///     - failure: returns StatusCode
pub fn raw_call<TFunction, TStatusCode, T>(function: TFunction) -> Result<T, TStatusCode>
where
    TFunction: FnOnce(*mut T) -> TStatusCode,
    TStatusCode: HrLikeStatusCode,
{
    let mut value = unsafe { std::mem::zeroed::<T>() };
    let returned = function(&mut value);

    match HrLikeStatusCode::ok(returned) {
        true => Ok(value),
        false => Err(returned),
    }
}

/// calls a `function(..., *mut out_value) -> code`, transforming its return value into a `Result<ComPtr<T>,
/// std::io::Error>`.
///
/// # remarks.
///
/// this function provides you a pointer to a `*mut T`. the provided callback is expected update set this value to a
/// valid com object on success.
///
/// this pattern is very common in the windows api.
///
/// # return value.
///
/// if `function(..., *mut T)` was:
///     - success: returns ComPtr<T>
///     - failure: returns std::io::Error (GetLastError)
pub fn com_call<TFunction, TStatusCode, T>(function: TFunction) -> Result<ComPtr<T>, std::io::Error>
where
    TFunction: FnOnce(*mut *mut T) -> TStatusCode,
    TStatusCode: HrLikeStatusCode,
{
    com_raw_call(function).map_err(TStatusCode::error)
}

/// calls a `function(..., *mut out_value) -> code`, transforming its return value into a `Result<ComPtr<T>,
/// std::io::Error>`.
///
/// # remarks.
///
/// this function provides you a pointer to a `*mut T`. the provided callback is expected update set this value to a
/// valid com object on success.
///
/// this pattern is very common in the windows api.
///
/// # return value.
///
/// if `function(..., *mut T)` was:
///     - success: returns ComPtr<T>
///     - failure: returns TStatusCode
pub fn com_raw_call<TFunction, TStatusCode, T>(
    function: TFunction,
) -> Result<ComPtr<T>, TStatusCode>
where
    TFunction: FnOnce(*mut *mut T) -> TStatusCode,
    TStatusCode: HrLikeStatusCode,
{
    let mut value = std::ptr::null_mut();
    let returned = function(&mut value);

    match HrLikeStatusCode::ok(returned) {
        true => Ok(unsafe { ComPtr::new(value) }),
        false => Err(returned),
    }
}

/// a trait that describes how we treat return values in a hr-style function call.
pub trait HrLikeStatusCode: Copy {
    fn ok(self: Self) -> bool;
    fn error(self: Self) -> std::io::Error;
}

impl HrLikeStatusCode for HRESULT {
    fn ok(self: HRESULT) -> bool {
        SUCCEEDED(self)
    }

    fn error(self: HRESULT) -> std::io::Error {
        std::io::Error::from_raw_os_error(self)
    }
}

impl HrLikeStatusCode for bool {
    fn ok(self: bool) -> bool {
        self
    }

    fn error(self: bool) -> std::io::Error {
        std::io::Error::last_os_error()
    }
}

impl HrLikeStatusCode for () {
    fn ok(self: ()) -> bool {
        true
    }

    fn error(self: ()) -> std::io::Error {
        unreachable!();
    }
}
