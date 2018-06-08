use super::{IoAction, IoDirection};
use kvm_sys as sys;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IoAddress {
    Memory(u64),
    Port(u64),
}

impl IoAddress {
    pub(super) fn address(&self) -> u64 {
        match self {
            IoAddress::Memory(v) => *v,
            IoAddress::Port(v) => *v,
        }
    }

    pub(super) fn flags(&self) -> u32 {
        match self {
            IoAddress::Memory(_) => 0,
            IoAddress::Port(_) => sys::KVM_IOEVENTFD_FLAG_PIO,
        }
    }

    pub fn ins(&self, size: usize) -> IoAction {
        IoAction(*self, IoDirection::In, size)
    }

    pub fn outs(&self, size: usize) -> IoAction {
        IoAction(*self, IoDirection::Out, size)
    }

    pub fn inb(&self) -> IoAction {
        self.ins(1)
    }

    pub fn outb(&self) -> IoAction {
        self.outs(1)
    }

    pub fn inw(&self) -> IoAction {
        self.ins(4)
    }

    pub fn outw(&self) -> IoAction {
        self.outs(4)
    }
}

macro_rules! addr_ops {
    ($v:ty) => {
        impl ops::Add<$v> for IoAddress {
            type Output = IoAddress;
            fn add(self, rhs: $v) -> IoAddress {
                match self {
                    IoAddress::Port(v) => IoAddress::Port(v + rhs as u64),
                    IoAddress::Memory(m) => IoAddress::Memory(m + rhs as u64),
                }
            }
        }

        impl ops::Sub<$v> for IoAddress {
            type Output = IoAddress;
            fn sub(self, rhs: $v) -> IoAddress {
                match self {
                    IoAddress::Port(v) => IoAddress::Port(v - rhs as u64),
                    IoAddress::Memory(m) => IoAddress::Memory(m - rhs as u64),
                }
            }
        }
    };

    ($head:ty $(, $tail:ty)*) => {
        addr_ops!($head);
        $(addr_ops!($tail);)*
    };
}

addr_ops![u64, u32, u16, u8, i64, i32, i16, i8];
