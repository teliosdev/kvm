use super::error::*;
use kvm_sys as kvm;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

#[derive(Debug)]
pub struct Core(pub(crate) File);

impl AsRawFd for Core {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for Core {
    unsafe fn from_raw_fd(fd: RawFd) -> Core {
        Core(File::from_raw_fd(fd))
    }
}

impl IntoRawFd for Core {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}
