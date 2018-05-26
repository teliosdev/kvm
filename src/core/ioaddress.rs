use kvm_sys as sys;

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
}
