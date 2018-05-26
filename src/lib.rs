#[macro_use]
extern crate error_chain;
extern crate mio;
extern crate tokio;

extern crate byteorder;
pub extern crate kvm_sys;
extern crate nix;

pub mod capability;
pub mod core;
mod error;
pub mod eventfd;
pub mod machine;
pub mod memory;
pub mod system;

pub use self::core::Core;
pub use self::error::Error;
pub use self::machine::Machine;
pub use self::system::System;
pub use kvm_sys as sys;

use self::error::*;

fn trail_mix<C, E>(elements: Vec<E>) -> Result<Box<C>> {
    use nix::libc::{c_void, malloc};
    use std::mem::size_of;

    unsafe {
        let bytes = size_of::<C>() + size_of::<E>() * elements.len();
        let alloc = malloc(bytes);

        if alloc == (0 as *mut c_void) {
            Err(ErrorKind::MemoryAllocationError.into())
        } else {
            let estart = ((alloc as usize) + size_of::<C>()) as *mut E;

            for (i, element) in elements.into_iter().enumerate() {
                *(((estart as usize) + i * size_of::<E>()) as *mut E) = element;
            }

            Ok(Box::from_raw(alloc as *mut C))
        }
    }
}
