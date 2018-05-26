use super::error::*;
use super::machine;
use kvm_sys as sys;
use nix;
use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

pub struct System(RawFd);

impl System {
    pub fn new() -> Result<System> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .chain_err(|| ErrorKind::KvmSystemOpenError)
            .map(|f| System(f.into_raw_fd()))
    }

    pub fn api_version(&self) -> Result<i32> {
        unsafe { sys::kvm_get_api_version(self.0) }
            .chain_err(|| ErrorKind::KvmSystemOperationError("kvm_get_api_version"))
    }

    pub fn mmap_bytes(&mut self) -> Result<i32> {
        unsafe { sys::kvm_get_vcpu_mmap_size(self.0) }
            .chain_err(|| ErrorKind::KvmSystemOperationError("kvm_get_vcpu_mmap_size"))
    }

    pub fn create_machine(&mut self, kind: i32) -> Result<machine::Machine> {
        unsafe { sys::kvm_create_vm(self.0, kind) }
            .chain_err(|| ErrorKind::KvmSystemOperationError("kvm_create_vm"))
            .and_then(move |fd| machine::Machine::new(fd, self.mmap_bytes()? as usize))
    }
}

impl AsRawFd for System {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Drop for System {
    fn drop(&mut self) -> () {
        nix::unistd::close(self.0).unwrap()
    }
}
