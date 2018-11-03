use super::Machine;
use byteorder::{ByteOrder, NativeEndian};
use error::*;
use kvm_sys as kvm;
use mio::event::Evented;
use mio::unix::EventedFd;
use mio::{Poll, PollOpt, Ready, Token};
use std::fs::File;
use std::io::{self, Read};
use std::ops::Drop;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use tokio::prelude::*;
use tokio::reactor::PollEvented2;

bitflags! {
    pub struct IoEventFdFlag: u32 {
        const PIO = kvm::KVM_IOEVENTFD_FLAG_PIO;
        const DEASSIGN = kvm::KVM_IOEVENTFD_FLAG_DEASSIGN;
        const DATAMATCH = kvm::KVM_IOEVENTFD_FLAG_DATAMATCH;
        const VIRTIO_CCW_NOTIFY = kvm::KVM_IOEVENTFD_FLAG_VIRTIO_CCW_NOTIFY;
    }
}

pub struct IoEventFd<'m> {
    pub(super) machine: &'m Machine,
    pub(super) file: File,
    pub(super) address: u64,
    pub(super) length: u32,
    pub(super) data: u64,
    pub(super) flags: IoEventFdFlag,
}

impl<'m> IoEventFd<'m> {
    pub(super) fn build() -> Result<File> {
        use nix::sys::eventfd;
        eventfd::eventfd(0, eventfd::EfdFlags::EFD_NONBLOCK)
            .map(|v| unsafe { File::from_raw_fd(v) })
            .chain_err(|| ErrorKind::CreateIoEventFdError)
    }

    pub fn read_value(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)
            .chain_err(|| ErrorKind::ReadIoEventFdError)?;
        Ok(NativeEndian::read_u64(&buf))
    }

    pub fn stream(&mut self) -> IoEventStream {
        unimplemented!()
    }
}

impl<'m> AsRawFd for IoEventFd<'m> {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl<'m> AsRef<File> for IoEventFd<'m> {
    fn as_ref(&self) -> &File {
        &self.file
    }
}

impl<'m> Drop for IoEventFd<'m> {
    fn drop(&mut self) {
        let _ = self.machine.ioeventfd_mod(
            self.address,
            self.length,
            self.data,
            self.flags | IoEventFdFlag::DEASSIGN,
            self.as_raw_fd(),
        );
    }
}

impl<'m> Evented for IoEventFd<'m> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl<'r, 'm: 'r> Evented for &'r mut IoEventFd<'m> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl<'m> Read for IoEventFd<'m> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.file.read_exact(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.file.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.file.read_to_string(buf)
    }
}

pub struct IoEventStream<'m, 's: 'm> {
    ev: PollEvented2<&'m mut IoEventFd<'s>>,
    buf: [u8; 8],
    len: usize,
}

impl<'m, 's: 'm> Stream for IoEventStream<'m, 's> {
    type Item = u64;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>> {
        let read_result = self
            .ev
            .poll_read(&mut self.buf[self.len..])
            .chain_err(|| ErrorKind::ReadIoEventFdError)?;

        match read_result {
            Async::Ready(v) => {
                self.len += v;
                if self.len == 8 {
                    let value = NativeEndian::read_u64(&self.buf);
                    self.len = 0;
                    Ok(Async::Ready(Some(value)))
                } else {
                    Ok(Async::NotReady)
                }
            }

            _ => Ok(Async::NotReady),
        }
    }
}
