use std::panic::AssertUnwindSafe;


/// invokes a closure, aborting the process if a panic occurs.
pub fn catch_abort<TFunction, TReturn>(function: TFunction) -> TReturn
where
    TFunction: FnOnce() -> TReturn,
{
    // "unwind safe" because we are immediately aborting after panic.
    std::panic::catch_unwind(AssertUnwindSafe(function)).unwrap_or_else(|_| std::process::abort())
}


/// a contiguous slice of `T` pointed to by `source`. ffi compatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Bunch<T> {
    pub source: *mut T,
}

impl<T> Bunch<T> {
    pub fn new(source: *mut T) -> Bunch<T> {
        Bunch { source }
    }

    pub unsafe fn get(&self, index: usize) -> Option<&T> {
        match !self.source.is_null() {
            true => Some(&*self.source.add(index)),
            false => None,
        }
    }

    pub unsafe fn get_mut(&self, index: usize) -> Option<&mut T> {
        match !self.source.is_null() {
            true => Some(&mut *self.source.add(index)),
            false => None,
        }
    }

    pub unsafe fn slice(&self, count: usize) -> Option<&[T]> {
        match !self.source.is_null() {
            true => Some(std::slice::from_raw_parts(self.source, count)),
            false => None,
        }
    }

    pub unsafe fn slice_mut(&self, count: usize) -> Option<&mut [T]> {
        match !self.source.is_null() {
            true => Some(std::slice::from_raw_parts_mut(self.source, count)),
            false => None,
        }
    }

    pub unsafe fn iter(&self, count: usize) -> Option<impl Iterator<Item = &T>> {
        self.slice(count).map(|x| x.into_iter())
    }

    pub unsafe fn iter_mut(&self, count: usize) -> Option<impl Iterator<Item = &mut T>> {
        self.slice_mut(count).map(|x| x.into_iter())
    }
}
