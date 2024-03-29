// sip implementation ripped out of `libcore` since it's been removed / deprecated.

use std::hash::Hasher;
use std::marker::PhantomData;

/// an implementation of siphash 2-4.
///
/// see: <https://131002.net/siphash/>
///
/// siphash is a general-purpose hashing function: it runs at a good speed (competitive with spooky and city) and
/// permits strong _keyed_ hashing. this lets you key your hashtables from a strong rng, such as
/// [`rand::os::OsRng`](https://doc.rust-lang.org/rand/rand/os/struct.OsRng.html).
///
/// although the siphash algorithm is considered to be generally strong, it is not intended for cryptographic purposes.
/// as such, all cryptographic uses of this implementation are *strongly discouraged*.
#[derive(Debug, Clone, Default)]
pub struct SipHasher24 {
    hasher: HashBase<Sip24Rounds>,
}

#[derive(Debug, Clone, Default)]
pub struct SipHasher13 {
    hasher: HashBase<Sip13Rounds>,
}

#[derive(Debug)]
struct HashBase<S: Sip> {
    k0: u64,
    k1: u64,
    length: usize, // how many bytes we've processed
    state: State,  // hash state
    tail: u64,     // unprocessed bytes le
    ntail: usize,  // how many bytes in tail are valid
    marker: PhantomData<S>,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct State {
    // v0, v2 and v1, v3 show up in pairs in the algorithm, and simd implementations of SipHash will use vectors of v02
    // and v13. By placing them in this order in the struct, the compiler can pick up on just a few simd optimizations
    // by itself.
    v0: u64,
    v2: u64,
    v1: u64,
    v3: u64,
}

macro_rules! compress {
    ($state: expr) => {{
        compress!($state.v0, $state.v1, $state.v2, $state.v3)
    }};

    ($v0: expr, $v1: expr, $v2: expr, $v3: expr) => {{
        $v0 = $v0.wrapping_add($v1);
        $v1 = $v1.rotate_left(13);
        $v1 ^= $v0;
        $v0 = $v0.rotate_left(32);
        $v2 = $v2.wrapping_add($v3);
        $v3 = $v3.rotate_left(16);
        $v3 ^= $v2;
        $v0 = $v0.wrapping_add($v3);
        $v3 = $v3.rotate_left(21);
        $v3 ^= $v0;
        $v2 = $v2.wrapping_add($v1);
        $v1 = $v1.rotate_left(17);
        $v1 ^= $v2;
        $v2 = $v2.rotate_left(32);
    }};
}

/// load an integer of the desired type from a byte stream, in le order. uses
/// `copy_nonoverlapping` to let the compiler generate the most efficient way
/// to load it from a possibly unaligned address.
///
/// unsafe because: unchecked indexing at i..i+size_of(int_ty)
macro_rules! load_int_le {
    ($buf: expr, $i: expr, $int_ty:ident) => {{
        debug_assert!($i + std::mem::size_of::<$int_ty>() <= $buf.len());

        let mut data = 0 as $int_ty;

        std::ptr::copy_nonoverlapping(
            $buf.get_unchecked($i),
            &mut data as *mut _ as *mut u8,
            std::mem::size_of::<$int_ty>(),
        );

        data.to_le()
    }};
}

/// load an u64 using up to 7 bytes of a byte slice.
///
/// unsafe because: unchecked indexing at start..start+len
#[inline]
unsafe fn u8to64_le(buf: &[u8], start: usize, len: usize) -> u64 {
    debug_assert!(len < 8);

    // current byte index (from lsb) in the output u64
    let mut i = 0;
    let mut out = 0;

    if i + 3 < len {
        out = load_int_le!(buf, start + i, u32) as u64;
        i += 4;
    }

    if i + 1 < len {
        out |= (load_int_le!(buf, start + i, u16) as u64) << (i * 8);
        i += 2
    }

    if i < len {
        out |= (*buf.get_unchecked(start + i) as u64) << (i * 8);
        i += 1;
    }

    debug_assert_eq!(i, len);
    out
}

impl SipHasher24 {
    /// creates a new `siphasher24` with the two initial keys set to 0.
    #[inline]
    pub fn new() -> SipHasher24 {
        SipHasher24::new_with_keys(0, 0)
    }

    /// creates a `siphasher24` that is keyed off the provided keys.
    #[inline]
    pub fn new_with_keys(key0: u64, key1: u64) -> SipHasher24 {
        SipHasher24 {
            hasher: HashBase::new_with_keys(key0, key1),
        }
    }
}

impl SipHasher13 {
    /// creates a new `siphasher13` with the two initial keys set to 0.
    #[inline]
    pub fn new() -> SipHasher13 {
        SipHasher13::new_with_keys(0, 0)
    }

    /// creates a `siphasher13` that is keyed off the provided keys.
    #[inline]
    pub fn new_with_keys(key0: u64, key1: u64) -> SipHasher13 {
        SipHasher13 {
            hasher: HashBase::new_with_keys(key0, key1),
        }
    }
}

impl<S: Sip> HashBase<S> {
    #[inline]
    fn new_with_keys(key0: u64, key1: u64) -> HashBase<S> {
        let mut state = HashBase {
            k0: key0,
            k1: key1,
            length: 0,
            state: State {
                v0: 0,
                v1: 0,
                v2: 0,
                v3: 0,
            },
            tail: 0,
            ntail: 0,
            marker: PhantomData,
        };
        state.reset();
        state
    }

    #[inline]
    fn reset(&mut self) {
        self.length = 0;
        self.state.v0 = self.k0 ^ 0x736f6d6570736575;
        self.state.v1 = self.k1 ^ 0x646f72616e646f6d;
        self.state.v2 = self.k0 ^ 0x6c7967656e657261;
        self.state.v3 = self.k1 ^ 0x7465646279746573;
        self.ntail = 0;
    }

    // specialized write function that is only valid for buffers with len <= 8.
    //
    // it's used to force inlining of write_u8 and write_usize, those would normally be inlined except for composite
    // types (that includes slices and str hashing because of delimiter). without this extra push the compiler is very
    // reluctant to inline delimiter writes, degrading performance substantially for the most common use cases.
    #[inline]
    fn short_write(&mut self, msg: &[u8]) {
        debug_assert!(msg.len() <= 8);

        let length = msg.len();
        self.length += length;

        let needed = 8 - self.ntail;
        let fill = std::cmp::min(length, needed);

        if fill == 8 {
            self.tail = unsafe { load_int_le!(msg, 0, u64) };
        } else {
            self.tail |= unsafe { u8to64_le(msg, 0, fill) } << (8 * self.ntail);
            if length < needed {
                self.ntail += length;
                return;
            }
        }

        self.state.v3 ^= self.tail;
        S::c_rounds(&mut self.state);
        self.state.v0 ^= self.tail;

        // buffered tail is now flushed, process new input.
        self.ntail = length - needed;
        self.tail = unsafe { u8to64_le(msg, needed, self.ntail) };
    }
}

impl Hasher for SipHasher24 {
    #[inline]
    fn write(&mut self, data: &[u8]) {
        self.hasher.write(data)
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }
}

impl Hasher for SipHasher13 {
    #[inline]
    fn write(&mut self, data: &[u8]) {
        self.hasher.write(data)
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }
}

impl<S: Sip> Hasher for HashBase<S> {
    // see short_write comment for explanation
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.short_write(unsafe {
            std::slice::from_raw_parts(
                &i as *const usize as *const u8,
                std::mem::size_of::<usize>(),
            )
        });
    }

    // see short_write comment for explanation
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.short_write(&[i]);
    }

    #[inline]
    fn write(&mut self, data: &[u8]) {
        let length = data.len();
        self.length += length;

        let mut needed = 0;

        if self.ntail != 0 {
            needed = 8 - self.ntail;
            self.tail |=
                unsafe { u8to64_le(data, 0, std::cmp::min(length, needed)) << 8 * self.ntail };

            if length < needed {
                self.ntail += length;
                return;
            } else {
                self.state.v3 ^= self.tail;
                S::c_rounds(&mut self.state);
                self.state.v0 ^= self.tail;
                self.ntail = 0;
            }
        }

        // buffered tail is now flushed, process new input.
        let len = length - needed;
        let left = len & 0x7;

        let mut i = needed;
        while i < len - left {
            let mi = unsafe { load_int_le!(data, i, u64) };

            self.state.v3 ^= mi;
            S::c_rounds(&mut self.state);
            self.state.v0 ^= mi;

            i += 8;
        }

        self.tail = unsafe { u8to64_le(data, i, left) };
        self.ntail = left;
    }

    #[inline]
    fn finish(&self) -> u64 {
        let mut state = self.state;

        let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail;

        state.v3 ^= b;
        S::c_rounds(&mut state);
        state.v0 ^= b;

        state.v2 ^= 0xff;
        S::d_rounds(&mut state);

        state.v0 ^ state.v1 ^ state.v2 ^ state.v3
    }
}

impl<S: Sip> Clone for HashBase<S> {
    #[inline]
    fn clone(&self) -> HashBase<S> {
        HashBase {
            k0: self.k0,
            k1: self.k1,
            length: self.length,
            state: self.state,
            tail: self.tail,
            ntail: self.ntail,
            marker: self.marker,
        }
    }
}

impl<S: Sip> Default for HashBase<S> {
    /// creates a `hasher<s>` with the two initial keys set to 0.
    #[inline]
    fn default() -> HashBase<S> {
        HashBase::new_with_keys(0, 0)
    }
}

trait Sip {
    fn c_rounds(_: &mut State);
    fn d_rounds(_: &mut State);
}

#[derive(Debug, Clone, Default)]
struct Sip13Rounds;

impl Sip for Sip13Rounds {
    #[inline]
    fn c_rounds(state: &mut State) {
        compress!(state);
    }

    #[inline]
    fn d_rounds(state: &mut State) {
        compress!(state);
        compress!(state);
        compress!(state);
    }
}

#[derive(Debug, Clone, Default)]
struct Sip24Rounds;

impl Sip for Sip24Rounds {
    #[inline]
    fn c_rounds(state: &mut State) {
        compress!(state);
        compress!(state);
    }

    #[inline]
    fn d_rounds(state: &mut State) {
        compress!(state);
        compress!(state);
        compress!(state);
        compress!(state);
    }
}
