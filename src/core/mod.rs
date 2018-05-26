use super::error::*;
use super::memory;
use super::trail_mix;
use kvm_sys as sys;
use std::os::unix::io::{AsRawFd, RawFd};

mod ioaddress;
mod ioeventfd;
mod mpstate;
mod pause;
pub use self::ioaddress::IoAddress;
pub use self::ioeventfd::IoEventFd;
pub use self::mpstate::MpState;
pub use self::pause::{Direction as IoDirection, Pause};
pub use kvm_sys::x86::{
    Dtable as DescriptorTable, Regs as Registers, Segment, Sregs as SpecialRegisters,
};
pub use kvm_sys::CpuIdEntry;

pub struct Core {
    pub id: i32,
    pub value: memory::Slab,
    fd: RawFd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TranslatedAddress {
    pub linear: u64,
    pub physical: u64,
    pub valid: bool,
    pub writable: bool,
    pub usermode: bool,
}

fn merge_into(run: sys::Run, pause: Pause) -> sys::Run {
    let (code, exit): (u32, sys::Exit) = pause.into();
    sys::Run {
        request_interrupt_window: run.request_interrupt_window,
        immediate_exit: run.immediate_exit,
        _pad1: [0; 6],
        exit_reason: code,
        ready_for_interrupt_injection: run.ready_for_interrupt_injection,
        if_flag: run.if_flag,
        flags: run.flags,
        cr8: run.cr8,
        apic_base: run.apic_base,
        exit: exit,
        kvm_valid_regs: run.kvm_valid_regs,
        kvm_dirty_regs: run.kvm_dirty_regs,
        _pad2: [0; 2048],
    }
}

impl Core {
    pub(super) fn new(id: i32, fd: RawFd, size: usize) -> Result<Core> {
        Ok(Core {
            id,
            value: memory::Slab::from_file(fd, 0, size)?,
            fd,
        })
    }

    pub fn mp_state(&self) -> Result<MpState> {
        let mut state = sys::MpState { mp_state: 0 };
        unsafe { sys::kvm_get_mp_state(self.fd, &mut state as *mut sys::MpState) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_get_mp_state"))
            .map(|v| MpState::from(v as u32))
    }

    pub fn set_mp_state(&self, mp_state: MpState) -> Result<()> {
        let state = sys::MpState {
            mp_state: mp_state.into(),
        };
        unsafe { sys::kvm_set_mp_state(self.fd, &state as *const sys::MpState) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_set_mp_state"))
            .map(|_| ())
    }

    pub fn interrupt(&self, irq: u32) -> Result<()> {
        let interrupt = sys::Interrupt { irq };
        unsafe { sys::kvm_interrupt(self.fd, &interrupt as *const sys::Interrupt) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_interrupt"))
            .map(|_| ())
    }

    pub fn run(&mut self) -> Result<()> {
        unsafe { sys::kvm_run(self.fd) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_run"))
            .map(|_| ())
    }

    pub fn get_run(&mut self) -> sys::Run {
        *self.value.as_ref()
    }

    pub fn set_pause(&mut self, pause: Pause) -> () {
        let new_run = merge_into(*self.value.as_ref(), pause);
        *self.value.as_mut() = new_run;
    }

    pub fn pause(&self) -> Pause {
        <memory::Slab as AsRef<sys::Run>>::as_ref(&self.value).into()
    }

    pub fn set_registers(&mut self, registers: Registers) -> Result<()> {
        unsafe { sys::x86::kvm_set_regs(self.fd, &registers as *const Registers) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_set_regs"))
            .map(|_| ())
    }

    pub fn registers(&mut self) -> Result<Registers> {
        let mut registers = unsafe { ::std::mem::uninitialized::<Registers>() };
        unsafe { sys::x86::kvm_get_regs(self.fd, &mut registers as *mut Registers) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_get_regs"))
            .map(|_| registers)
    }

    pub fn set_special_registers(&mut self, registers: SpecialRegisters) -> Result<()> {
        unsafe { sys::x86::kvm_set_sregs(self.fd, &registers as *const SpecialRegisters) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_set_sregs"))
            .map(|_| ())
    }

    pub fn special_registers(&mut self) -> Result<SpecialRegisters> {
        let mut registers = unsafe { ::std::mem::uninitialized::<SpecialRegisters>() };
        unsafe { sys::x86::kvm_get_sregs(self.fd, &mut registers as *mut SpecialRegisters) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_get_sregs"))
            .map(|_| registers)
    }

    pub fn set_cpuids<C: AsRef<[CpuIdEntry]>>(&mut self, cpuids: C) -> Result<()> {
        let entries = cpuids.as_ref().to_owned();
        let count = entries.len();
        let mut cpuid: Box<sys::CpuId> = trail_mix(entries)?;
        cpuid.nent = count as u32;
        cpuid.padding = 0;

        unsafe { sys::kvm_set_cpuid(self.fd, &*cpuid as *const sys::CpuId) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_set_cpuid"))
            .map(|_| ())
    }

    pub fn create_ioeventfd(&mut self, addr: IoAddress, len: u32) -> Result<IoEventFd> {
        let eventfd = IoEventFd::new(addr, len)?;
        let ioeventfd = sys::IoEventFd {
            datamatch: 0,
            addr: addr.address(),
            len,
            fd: eventfd.as_raw_fd(),
            flags: addr.flags(),
            _pad: [0; 36],
        };

        unsafe { sys::kvm_ioeventfd(self.fd, &ioeventfd as *const sys::IoEventFd) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_ioeventfd"))
            .map(|_| eventfd)
    }

    pub fn remove_ioeventfd(&mut self, eventfd: &IoEventFd) -> Result<()> {
        let ioeventfd = sys::IoEventFd {
            datamatch: 0,
            addr: eventfd.addr.address(),
            len: eventfd.len,
            fd: eventfd.as_raw_fd(),
            flags: eventfd.addr.flags() | sys::KVM_IOEVENTFD_FLAG_DEASSIGN,
            _pad: [0; 36],
        };

        unsafe { sys::kvm_ioeventfd(self.fd, &ioeventfd as *const sys::IoEventFd) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_ioeventfd"))
            .map(|_| ())
    }

    pub fn translate(&mut self, addr: u64) -> Result<TranslatedAddress> {
        let mut translation = sys::Translation {
            linear_address: addr,
            physical_address: 0,
            valid: 0,
            writable: 0,
            usermode: 0,
            _pad: [0; 5],
        };

        unsafe { sys::kvm_translate(self.fd, &mut translation as *mut sys::Translation) }
            .chain_err(|| ErrorKind::KvmCoreOperationError("kvm_translate"))
            .map(|_| TranslatedAddress {
                linear: addr,
                physical: translation.physical_address,
                valid: translation.valid != 0,
                writable: translation.writable != 0,
                usermode: translation.usermode != 0,
            })
    }
}

impl AsRawFd for Core {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        use nix;
        nix::unistd::close(self.fd).unwrap()
    }
}

unsafe impl Send for Core {}
