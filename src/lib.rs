#![feature(optin_builtin_traits)]
#![feature(allocator_api)]

extern crate kvm_sys;
#[macro_use]
extern crate error_chain;
extern crate nix;
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate mio;
extern crate tokio;

pub mod core;
mod error;
pub mod machine;
pub mod system;

pub use self::error::{Error, ErrorKind};
