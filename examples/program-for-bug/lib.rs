#![no_std]
#![no_main]

extern crate core;

use polkavm_derive::polkavm_export;
use primitive_types::U256;

mod bump;

#[global_allocator]
static mut ALLOC: bump::BumpAllocator = bump::BumpAllocator {};

#[polkavm_derive::polkavm_import]
extern "C" {
    pub fn value_transferred(output: *const u8);
}


/// A static buffer of variable capacity.
pub struct StaticBuffer {
    /// A static buffer of variable capacity.
    buffer: [u8; Self::CAPACITY],
}

impl StaticBuffer {
    /// The capacity of the static buffer.
    /// Usually set to 16 kB.
    /// Can be modified by setting `INK_STATIC_BUFFER_SIZE` environmental variable.
    const CAPACITY: usize = 4096;

    /// Creates a new static buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [0; Self::CAPACITY],
        }
    }
}

impl<'a> ScopedBuffer<'a> {
    /// Splits the scoped buffer into yet another piece to operate on it temporarily.
    ///
    /// The split buffer will have an offset of 0 but be offset by `self`'s offset.
    pub fn split(&mut self) -> ScopedBuffer {
        ScopedBuffer {
            offset: 0,
            buffer: &mut self.buffer[self.offset..],
        }
    }

    /// Returns the first `len` bytes of the buffer as mutable slice.
    pub fn take(&mut self, len: usize) -> &'a mut [u8] {
        debug_assert_eq!(self.offset, 0);
        debug_assert!(len <= self.buffer.len());
        let len_before = self.buffer.len();
        let buffer = core::mem::take(&mut self.buffer);
        let (lhs, rhs) = buffer.split_at_mut(len);
        self.buffer = rhs;
        debug_assert_eq!(lhs.len(), len);
        let len_after = self.buffer.len();
        debug_assert_eq!(len_before.checked_sub(len_after).unwrap(), len);
        lhs
    }
}

impl core::ops::Index<core::ops::RangeFull> for StaticBuffer {
    type Output = [u8];

    #[inline(always)]
    fn index(&self, index: core::ops::RangeFull) -> &Self::Output {
        core::ops::Index::index(&self.buffer[..], index)
    }
}

impl core::ops::IndexMut<core::ops::RangeFull> for StaticBuffer {
    #[inline(always)]
    fn index_mut(&mut self, index: core::ops::RangeFull) -> &mut Self::Output {
        core::ops::IndexMut::index_mut(&mut self.buffer[..], index)
    }
}

/// Scoped access to an underlying bytes buffer.
///
/// # Note
///
/// This is used to efficiently chunk up ink!'s internal static 16 kB buffer
/// into smaller sub buffers for processing different parts of computations.
#[derive(Debug)]
pub struct ScopedBuffer<'a> {
    offset: usize,
    buffer: &'a mut [u8],
}

impl<'a> From<&'a mut [u8]> for ScopedBuffer<'a> {
    fn from(buffer: &'a mut [u8]) -> Self {
        Self { offset: 0, buffer }
    }
}

#[polkavm_export(abi = polkavm_derive::default_abi)]
pub fn deploy() {
    // (1) This will result in a trap with
    // `Indirect jump to dynamic address 4294835888: invalid (bad jump table index)`
    unsafe {
        let u256 = [0u8; 32];
        value_transferred(u256.as_ptr());

        let val = U256::from_little_endian(&u256[..]);
        assert_eq!(val, U256::zero());
    }

    // (2) Uncomment + compile without `--release`.
    // This will result in a trap with
    // `Store of 8 bytes to 0xfffdddf0 failed! (pc = 336, cycle = 33)`
    /*
    #[allow(dead_code)]
    fn scoped_buffer<'a>(buffer: &'a mut StaticBuffer) -> ScopedBuffer<'a> {
        ScopedBuffer::from(&mut buffer[..])
    }
    unsafe {
        let mut buffer = StaticBuffer::new();
        let mut scope = scoped_buffer(&mut buffer);
        let mut u256: [u8; 32] = scope.take(32).try_into().unwrap();

        let u256 = [0u8; 32];
        value_transferred(u256.as_ptr());

        let val = U256::from_little_endian(&u256[..]);
        assert_eq!(val, U256::zero());
    }
    */
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked();
    }
}
