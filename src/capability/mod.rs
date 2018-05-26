use super::error::*;
use kvm_sys as sys;
use std::os::unix::io::AsRawFd;

pub trait Capability {
    fn check_capability(&self, kind: CapabilityKind) -> Result<i32>;
    fn enable_capability(&self, kind: CapabilityKind) -> Result<()>;

    fn ensure_capability(&self, kind: CapabilityKind) -> Result<()> {
        if self.check_capability(kind)? != 1 {
            Err(ErrorKind::KvmCapabilityFailError(kind).into())
        } else {
            Ok(())
        }
    }
}

impl<T: AsRawFd> Capability for T {
    fn check_capability(&self, kind: CapabilityKind) -> Result<i32> {
        unsafe { sys::kvm_check_extension(self.as_raw_fd(), kind.into()) }
            .chain_err(|| ErrorKind::KvmCapabilityError("kvm_check_extension"))
    }

    fn enable_capability(&self, kind: CapabilityKind) -> Result<()> {
        let enable_cap = sys::EnableCap {
            cap: kind as i32,
            flags: 0,
            args: [0; 4],
            _pad: [0; 64],
        };
        unsafe { sys::kvm_enable_cap(self.as_raw_fd(), &enable_cap as *const sys::EnableCap) }
            .chain_err(|| ErrorKind::KvmCapabilityError("kvm_enable_cap"))
            .map(|_| ())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityKind {
    IoEventFd,
    IrqChip,
    IrqFd,
    SetTssAddr,
    SetIdentityMapAddr,
    MemorySlotCount,
}

impl Into<i32> for CapabilityKind {
    fn into(self) -> i32 {
        match self {
            CapabilityKind::IoEventFd => sys::KVM_CAP_IOEVENTFD,
            CapabilityKind::IrqChip => sys::KVM_CAP_IRQCHIP,
            CapabilityKind::IrqFd => sys::KVM_CAP_IRQFD,
            CapabilityKind::SetTssAddr => sys::KVM_CAP_SET_TSS_ADDR,
            CapabilityKind::SetIdentityMapAddr => sys::KVM_CAP_SET_IDENTITY_MAP_ADDR,
            CapabilityKind::MemorySlotCount => sys::KVM_CAP_NR_MEMSLOTS,
        }
    }
}
