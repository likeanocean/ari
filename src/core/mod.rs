use parking_lot::{Once, OnceState};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

static INITIALIZED: Once = Once::new();

/// initializes `ari`, setting up the environment for use.
pub fn initialize() {
    INITIALIZED.call_once(|| {
        crate::os::initialize();
    });
}

/// returns true if `ari` has been initialized.
pub fn initialized() -> bool {
    INITIALIZED.state() != OnceState::Done
}

/// keep `x`, preventing llvm from optimizing it away.
#[cfg(feature = "asm")]
pub fn keep<T>(x: T) -> T {
    unsafe { llvm_asm!["" : : "r"(&x)] }
    x
}

// asserts (at compile time) that some object is `Sync` or `Send`.
pub fn assert_send(_: impl Send) {}

pub fn assert_sync(_: impl Sync) {}

pub fn assert_send_sync(_: impl Send + Sync) {}

/// a trait that encompasses all types.
pub trait Any {}

impl<T> Any for T {}

/// an opaque error that be converted from any other error type.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlackHole;

impl<T> Into<Result<T, BlackHole>> for BlackHole {
    fn into(self) -> Result<T, BlackHole> {
        Err(BlackHole)
    }
}

impl Display for BlackHole {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(formatter, "BlackHole")
    }
}

impl<T> From<T> for BlackHole
where
    T: Error,
{
    fn from(_: T) -> BlackHole {
        BlackHole
    }
}

/// a wrapper type that ensures its wrapped item implements debug.
pub struct DefaultDebug<T>(pub T);

impl<T> Debug for DefaultDebug<T> {
    default fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match cfg!(verbose) {
            true => write!(formatter, "{}", std::any::type_name::<T>()),
            false => Ok(()),
        }
    }
}

impl<T> Debug for DefaultDebug<T>
where
    T: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<T> Deref for DefaultDebug<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for DefaultDebug<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// returns a slice as an array, if `byte_length(array) == byte_length(slice)`. this function is unsafe because `T` may
/// not be a memcopy-safe type.
///
/// # notes.
///
/// we do not require `T: Copy` because `T` might be very large and we do not want to force this trait on users.
pub unsafe fn slice_as_array<T, TArray>(data: &[T]) -> Option<&TArray>
where
    TArray: AsRef<[T]>,
{
    let a = std::mem::size_of::<TArray>();
    let b = std::mem::size_of::<T>() * data.len();

    match a == b {
        true => Some(&*(data.as_ptr() as *const TArray)),
        false => None,
    }
}

/// convert a singular `T` into a single element slice of `T`.
pub fn as_slice<T>(item: &T) -> &[T] {
    // safe: the memory layout of a singular `T` is always the same as an array of one `T`.
    unsafe { std::slice::from_raw_parts(item, 1) }
}

/// convert a singular `T` into a single element slice of `T`.
pub fn as_slice_mut<T>(item: &mut T) -> &mut [T] {
    // safe: the memory layout of a singular `T` is always the same as an array of one `T`.
    unsafe { std::slice::from_raw_parts_mut(item, 1) }
}

/// convert an `Option<T>` into a zero or single element slice of `T`.
pub fn option_as_slice<T>(value: &Option<T>) -> &[T] {
    match value {
        Some(x) => as_slice(x),
        None => &[],
    }
}

/// convert an `Option<T>` into a zero or single element slice of `T`.
pub fn option_as_slice_mut<T>(value: &mut Option<T>) -> &mut [T] {
    match value {
        Some(x) => as_slice_mut(x),
        None => &mut [],
    }
}

/// a bitfield.
///
/// this is c ffi compatible, which means for example a `[u8; 4]` bitfield consumes 4 bytes of space. just like a
/// u32-based c-style bitfield.
///
/// # examples.
///
/// ```
/// # #![feature(type_ascription)]
/// # use ari::BitField;
///
/// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
///
/// assert_eq!(x.value(), &[0, 0, 0, 0]);
///
/// x.set(0, true);
/// x.set(4, true);
/// x.set(8, true);
/// x.set(16, true);
/// x.set_value(20, 3, 7u64);
///
/// assert_eq!(x.get(0), true);
/// assert_eq!(x.get(4), true);
/// assert_eq!(x.get(8), true);
/// assert_eq!(x.get(16), true);
/// assert_eq!(x.get_value(20, 3): u64, 7);
///
/// assert_eq!(x.value(), &[17, 1, 113, 0]);
/// ```
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitField<TStorage, TAlignment>
where
    TStorage: AsRef<[u8]> + AsMut<[u8]>,
{
    storage: TStorage,
    alignment: [TAlignment; 0],
}

impl<TStorage, TAlignment> BitField<TStorage, TAlignment>
where
    TStorage: AsRef<[u8]> + AsMut<[u8]>,
{
    /// creates a new bitfield with the specified value.
    ///
    /// # examples.
    ///
    /// ```
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// assert_eq!(x.value(), &[0, 0, 0, 0]);
    /// ```
    #[inline]
    pub fn new(storage: TStorage) -> Self {
        BitField {
            storage,
            alignment: [],
        }
    }

    /// retrieves the bit at `index`.
    ///
    /// # examples.
    ///
    /// ```
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// assert_eq!(x.get(0), false);
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let storage = self.storage.as_ref();

        debug_assert![storage.len() >= index / 8];

        #[rustfmt::skip]
        let shift = if cfg!(target_endian = "little") { index % 8 } else { 7 - (index % 8) };
        let byte = storage[index / 8];
        let mask = 1 << shift;

        byte & mask != 0
    }

    /// places `value` at the specified bit `index`.
    ///
    /// # examples.
    ///
    /// ```
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// x.set(0, true);
    ///
    /// assert_eq!(x.get(0), true);
    /// ```
    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let storage = self.storage.as_mut();

        debug_assert![storage.len() >= index / 8];

        #[rustfmt::skip]
        let shift = if cfg!(target_endian = "little") { index % 8 } else { 7 - (index % 8) };
        let byte = &mut storage[index / 8];
        let mask = 1 << shift;

        match value {
            true => *byte |= mask,
            false => *byte &= !mask,
        }
    }

    /// retrieves the value at the specified bit range `[offset..offset + width]`.
    ///
    /// # examples.
    ///
    /// ```
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// x.set_value(0, 5, 0xfffffu64);
    //
    /// assert_eq!(x.get_value::<u64>(0, 5), 31);
    /// ```
    #[inline]
    pub fn get_value<T>(&self, offset: usize, width: usize) -> T
    where
        T: From<u64>,
    {
        let storage = self.storage.as_ref();

        debug_assert![width <= 64];
        debug_assert![storage.len() > (offset + width) / 8];

        let mut value = 0u64;

        for i in 0..width {
            if self.get(offset + i) {
                let shift = if cfg!(target_endian = "big") {
                    width - i - 1
                } else {
                    i
                };

                value |= 1 << shift;
            }
        }

        value.into()
    }

    /// places `value` at the specified bit range `[offset..offset + width]`.
    ///
    /// `value` is truncated if it exceeds the maximum representable value defined by `(offset, width)`.
    ///
    /// # examples.
    ///
    /// ## simple example.
    ///
    /// ```
    /// # #![feature(type_ascription)]
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// x.set_value(0, 5, 0xfffffu64);
    ///
    /// assert_eq!(x.get_value(0, 5): u64, 31);
    /// ```
    ///
    /// ## 4 bit value truncated to 2 bits.
    ///
    /// ```
    /// # #![feature(type_ascription)]
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// x.set_value(0, 2, 0b1111u64);
    ///
    /// assert_eq!(x.get_value(0, 4): u64, 0b11);
    /// ```
    #[inline]
    pub fn set_value<T>(&mut self, offset: usize, width: usize, value: T)
    where
        T: Into<u64>,
    {
        let storage = self.storage.as_ref();
        let value = Into::<u64>::into(value);

        debug_assert![width <= 64];
        debug_assert![storage.len() > (offset + width) / 8];

        for i in 0..width {
            let index = if cfg!(target_endian = "big") {
                width - i - 1
            } else {
                i
            };
            let mask = 1 << i;

            self.set(index + offset, value & mask != 0);
        }
    }

    /// returns the raw value of this bitfield.
    ///
    /// # examples.
    ///
    /// ```
    /// # use ari::BitField;
    ///
    /// let mut x: BitField<[u8; 4], u32> = BitField::new([0u8; 4]);
    ///
    /// x.set(0, true);
    /// x.set(4, true);
    /// x.set(8, true);
    /// x.set(16, true);
    /// x.set_value(20, 3, 9u64);
    ///
    /// assert_eq!(x.value(), &[17, 1, 17, 0]);
    /// ```
    #[inline]
    pub fn value(&self) -> &TStorage {
        &self.storage
    }
}

/// extensions to `std::vec::Vec`.
pub trait VecExt {
    /// clears this `Vec<t>`. if `T` is copy, this method optimizes the clear by truncating the vec's length to zero,
    /// unlike `Vec<T>::clear()`.
    fn clear_vec(&mut self);
}

impl<T> VecExt for Vec<T> {
    default fn clear_vec(&mut self) {
        self.clear();
    }
}

impl<T> VecExt for Vec<T>
where
    T: Copy,
{
    fn clear_vec(&mut self) {
        // safe: `T` is copy.
        unsafe {
            self.set_len(0);
        }
    }
}

/// extensions to `bool`.
pub trait BoolExt {
    /// converts a true into `Some(())` and false into `None`.
    fn as_option(&self) -> Option<()>;
}

impl BoolExt for bool {
    fn as_option(&self) -> Option<()> {
        match self {
            true => Some(()),
            false => None,
        }
    }
}
