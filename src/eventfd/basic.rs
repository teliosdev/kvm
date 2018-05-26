use mio::event::Evented;
use mio::unix::EventedFd;
use mio::{Poll, PollOpt, Ready, Token};
use nix;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
pub(crate) struct BasicEventFd(RawFd);

impl BasicEventFd {
    pub(super) fn new(fd: RawFd) -> BasicEventFd {
        BasicEventFd(fd)
    }
}

fn nix_to_io<T>(result: nix::Result<T>) -> io::Result<T> {
    result.map_err(|e| match e {
        nix::Error::Sys(errno) => io::Error::from_raw_os_error(errno as i32),
        nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidInput, e),
        nix::Error::InvalidUtf8 => io::Error::new(io::ErrorKind::InvalidData, e),
        nix::Error::UnsupportedOperation => io::Error::new(io::ErrorKind::Other, e),
    })
}

impl io::Read for BasicEventFd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        nix_to_io(nix::unistd::read(self.0, buf))
    }
}

impl io::Write for BasicEventFd {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        nix_to_io(nix::unistd::write(self.0, buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        nix_to_io(nix::unistd::fsync(self.0))
    }
}

impl AsRawFd for BasicEventFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Evented for BasicEventFd {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.0).deregister(poll)
    }
}

impl Drop for BasicEventFd {
    fn drop(&mut self) {
        nix::unistd::close(self.0).unwrap()
    }
}
