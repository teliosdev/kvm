use super::capability::*;
use super::core;
use super::error::*;
use super::memory;
use kvm_sys as sys;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::{Arc, Mutex};

pub struct Machine {
    fd: RawFd,
    msize: usize,
    pub maps: Vec<(u64, Arc<Mutex<memory::Slab>>, u64)>,
}

impl Machine {
    pub(super) fn new(fd: RawFd, msize: usize) -> Result<Machine> {
        Ok(Machine {
            fd,
            msize,
            maps: vec![],
        })
    }

    pub fn create_irqchip(&mut self) -> Result<()> {
        self.ensure_capability(CapabilityKind::IrqChip)?;
        unsafe { sys::kvm_create_irqchip(self.fd) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_create_irqchip"))
            .map(|_| ())
    }

    pub fn create_pit(&mut self) -> Result<()> {
        self.ensure_capability(CapabilityKind::Pit2)?;
        let pit2 = sys::PitConfig {
            flags: sys::KVM_PIT_SPEAKER_DUMMY,
            _pad: [0; 15],
        };
        unsafe { sys::kvm_create_pit2(self.fd, &pit2 as *const sys::PitConfig) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_create_pit2"))
            .map(|_| ())
    }

    pub fn set_irq_line(&mut self, line: u32, triggered: bool) -> Result<()> {
        self.ensure_capability(CapabilityKind::IrqChip)?;
        let level = sys::IrqLevel {
            irq: line,
            level: triggered as u32,
        };

        unsafe { sys::kvm_irq_line(self.fd, &level as *const sys::IrqLevel) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_irq_line"))
            .map(|_| ())
    }

    pub fn create_core(&mut self, id: i32) -> Result<core::Core> {
        unsafe { sys::kvm_create_vcpu(self.fd, id) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_create_vcpu"))
            .and_then(move |fd| core::Core::new(id, fd, self.msize))
    }

    pub fn clock(&mut self, stable: bool) -> Result<u64> {
        let mut clock_data = sys::ClockData {
            clock: 0,
            flags: if stable { sys::KVM_CLOCK_TSC_STABLE } else { 0 },
            _pad: [0; 9],
        };

        unsafe { sys::kvm_get_clock(self.fd, &mut clock_data as *mut sys::ClockData) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_get_clock"))
            .map(|_| clock_data.clock)
    }

    pub fn set_clock(&mut self, clock: u64) -> Result<()> {
        let clock_data = sys::ClockData {
            clock,
            flags: 0,
            _pad: [0; 9],
        };

        unsafe { sys::kvm_set_clock(self.fd, &clock_data as *const sys::ClockData) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_set_clock"))
            .map(|_| ())
    }

    pub fn set_identity_map_addr(&mut self, addr: Option<u64>) -> Result<()> {
        self.ensure_capability(CapabilityKind::SetIdentityMapAddr)?;
        let addr = addr.unwrap_or(0);

        unsafe { sys::kvm_set_identity_map_addr(self.fd, &addr as *const u64) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_set_identity_map_addr"))
            .map(|_| ())
    }

    pub fn set_tss_addr(&mut self, addr: Option<u32>) -> Result<()> {
        self.ensure_capability(CapabilityKind::SetTssAddr)?;
        let addr = addr.unwrap_or(0xfffbd000u32);

        unsafe { sys::kvm_set_tss_addr(self.fd, addr) }
            .chain_err(|| ErrorKind::KvmMachineOperationError("kvm_set_tss_addr"))
            .map(|_| ())
    }

    pub fn create_memory_region(
        &mut self,
        at: u64,
        size: usize,
    ) -> Result<Arc<Mutex<memory::Slab>>> {
        let slab = memory::Slab::from_anon(size)?;
        let arc = Arc::new(Mutex::new(slab));
        self.mount_memory_region(at, arc.clone())?;
        Ok(arc)
    }

    pub fn create_read_only_memory_region(
        &mut self,
        at: u64,
        size: usize,
    ) -> Result<Arc<Mutex<memory::Slab>>> {
        let slab = memory::Slab::from_anon(size)?;
        let arc = Arc::new(Mutex::new(slab));
        self.mount_read_only_memory_region(at, arc.clone())?;
        Ok(arc)
    }

    pub fn mount_memory_region(&mut self, at: u64, slab: Arc<Mutex<memory::Slab>>) -> Result<()> {
        let (address, len) = {
            let mslab = slab.lock().unwrap();
            (mslab.address(), mslab.len() as u64)
        };
        let mr = sys::UserspaceMemoryRegion {
            slot: (self.maps.len() + 1) as u32,
            flags: 0,
            guest_phys_addr: at,
            memory_size: len,
            userspace_addr: address,
        };

        unsafe {
            sys::kvm_set_user_memory_region(self.fd, &mr as *const sys::UserspaceMemoryRegion)
        }.chain_err(|| ErrorKind::KvmMachineOperationError("kvm_set_user_memory_region"))
            .map(move |_| self.maps.push((at, slab, len)))
    }

    pub fn mount_read_only_memory_region(
        &mut self,
        at: u64,
        slab: Arc<Mutex<memory::Slab>>,
    ) -> Result<()> {
        let (address, len) = {
            let mslab = slab.lock().unwrap();
            (mslab.address(), mslab.len() as u64)
        };

        let mr = sys::UserspaceMemoryRegion {
            slot: (self.maps.len() + 1) as u32,
            flags: sys::KVM_MEM_READONLY,
            guest_phys_addr: at,
            memory_size: len,
            userspace_addr: address,
        };

        unsafe {
            sys::kvm_set_user_memory_region(self.fd, &mr as *const sys::UserspaceMemoryRegion)
        }.chain_err(|| ErrorKind::KvmMachineOperationError("kvm_set_user_memory_region"))
            .map(move |_| self.maps.push((at, slab, len)))
    }

    pub fn locate(&self, addr: u64) -> Option<(Arc<Mutex<memory::Slab>>, u64)> {
        self.maps
            .iter()
            .find(|v| addr >= v.0 && addr < (v.0 + v.2))
            .map(|v| (v.1.clone(), addr - v.0))
    }
}

impl AsRawFd for Machine {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        use nix;
        nix::unistd::close(self.fd).unwrap()
    }
}
