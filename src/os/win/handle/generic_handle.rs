use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;



pub trait GenericHandleDtor<THandle>
where
    THandle: Copy,
{
    fn destroy(handle: THandle);
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenericHandle<THandle, TDestructor>
where
    THandle: Copy + Eq,
    TDestructor: GenericHandleDtor<THandle>,
{
    handle:  THandle,
    phantom: PhantomData<TDestructor>,
}

impl<THandle, TDestructor> GenericHandle<THandle, TDestructor>
where
    THandle: Copy + Eq,
    TDestructor: GenericHandleDtor<THandle>,
{
    pub fn new(handle: THandle) -> GenericHandle<THandle, TDestructor> {
        GenericHandle {
            handle,
            phantom: PhantomData,
        }
    }

    pub fn create(
        create: impl FnOnce() -> THandle,
        valid: impl FnOnce(THandle) -> bool,
    ) -> Result<GenericHandle<THandle, TDestructor>, std::io::Error> {
        let handle = create();

        match valid(handle) {
            true => Ok(GenericHandle {
                handle,
                phantom: PhantomData,
            }),
            false => Err(std::io::Error::last_os_error()),
        }
    }

    pub fn as_raw(&self) -> THandle {
        self.handle
    }
}

impl<THandle, TDestructor> Debug for GenericHandle<THandle, TDestructor>
where
    THandle: Debug + Copy + Eq,
    TDestructor: GenericHandleDtor<THandle>,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct("GenericHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<THandle, TDestructor> Drop for GenericHandle<THandle, TDestructor>
where
    THandle: Copy + Eq,
    TDestructor: GenericHandleDtor<THandle>,
{
    fn drop(&mut self) {
        TDestructor::destroy(self.handle)
    }
}
