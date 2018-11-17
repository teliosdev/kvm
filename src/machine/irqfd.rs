use super::Machine;
use byteorder::{ByteOrder, NativeEndian};
use error::{Error, ErrorKind, ResultExt};
use kvm_sys as kvm;
use mio::event::Evented;
use mio::unix::EventedFd;
use mio::{Poll, PollOpt, Ready, Token};
use std::fs::File;
use std::io::{self, Write as StdWrite};
use std::ops::Drop;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use tokio::io::Error as TokioIoError;
use tokio::prelude::*;

bitflags! {
    pub struct IrqFdFlag: u32 {
        /// Removes the IrqFd from the machine.  Do not use this.
        const DEASSIGN = kvm::KVM_IRQFD_FLAG_DEASSIGN;
        /// This operation is not supported by this library.  Please do
        /// not use it unless you know what you are doing.
        const RESAMPLE = kvm::KVM_IRQFD_FLAG_RESAMPLE;
    }
}

pub struct IrqFd<'m> {
    pub(super) machine: &'m Machine,
    pub(super) file: File,
    pub(super) gsi: u32,
    pub(super) flags: IrqFdFlag,
}

impl<'m> IrqFd<'m> {
    pub(super) fn build() -> Result<File, Error> {
        use nix::sys::eventfd;
        eventfd::eventfd(0, eventfd::EfdFlags::EFD_NONBLOCK)
            .map(|v| unsafe { File::from_raw_fd(v) })
            .chain_err(|| ErrorKind::CreateIrqFdError)
    }

    pub fn notify<'s: 'm>(&'s mut self) -> impl Future<Item = (), Error = Error> + 's + 'm {
        let mut data = [0u8; 8];
        NativeEndian::write_u64(&mut data, 1);
        ::tokio::io::write_all(self, data)
            .map(|_| ())
            .map_err(|err| Error::with_chain(err, ErrorKind::NotifyIrqFdError))
    }
}

impl<'m> Drop for IrqFd<'m> {
    fn drop(&mut self) {
        let _ =
            self.machine
                .irqfd_mod(self.gsi, self.flags | IrqFdFlag::DEASSIGN, self.as_raw_fd());
    }
}

impl<'m> AsyncWrite for IrqFd<'m> {
    fn shutdown(&mut self) -> Result<Async<()>, TokioIoError> {
        Ok(Async::Ready(()))
    }
}

impl<'m> Evented for IrqFd<'m> {
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

impl<'m> AsRawFd for IrqFd<'m> {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl<'m> AsRef<File> for IrqFd<'m> {
    fn as_ref(&self) -> &File {
        &self.file
    }
}

impl<'m> StdWrite for IrqFd<'m> {
    fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> ::std::io::Result<()> {
        self.file.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> ::std::io::Result<()> {
        self.file.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: ::std::fmt::Arguments) -> ::std::io::Result<()> {
        self.file.write_fmt(fmt)
    }
}
