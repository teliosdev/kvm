use super::error::*;
use byteorder::{ByteOrder, NativeEndian};
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::result::Result as StdResult;
use tokio::io as tio;
use tokio::prelude::*;
use tokio::reactor::Handle;
use tokio::reactor::PollEvented2;

mod basic;
use self::basic::BasicEventFd;

pub struct EventFd(pub(crate) PollEvented2<BasicEventFd>);

impl EventFd {
    pub fn new() -> Result<EventFd> {
        let fd = create_eventfd()?;
        let basic = BasicEventFd::new(fd);
        let polle = PollEvented2::new(basic);
        Ok(EventFd(polle))
    }

    pub fn new_with_handle(handle: &Handle) -> Result<EventFd> {
        let fd = create_eventfd()?;
        let basic = BasicEventFd::new(fd);
        let polle =
            PollEvented2::new_with_handle(basic, handle).chain_err(|| ErrorKind::TokioError)?;
        Ok(EventFd(polle))
    }
}

#[cfg(linux)]
fn create_eventfd() -> Result<RawFd> {
    use nix::sys::eventfd;
    eventfd::eventfd(0, eventfd::EfdFlags::empty())
        .chain_err(|| ErrorKind::KvmCoreOperationError("eventfd"))
}

#[cfg(not(linux))]
fn create_eventfd() -> Result<RawFd> {
    Err(ErrorKind::UnsupportedOsError.into())
}

impl io::Read for EventFd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Write for EventFd {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl tio::AsyncRead for EventFd {}
impl tio::AsyncWrite for EventFd {
    fn shutdown(&mut self) -> StdResult<Async<()>, tio::Error> {
        self.0.shutdown()
    }
}

impl stream::Stream for EventFd {
    type Item = u64;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<u64>>> {
        let mut buf = [0u8; 8];
        match self.poll_read(&mut buf) {
            Ok(Async::Ready(_)) => Ok(Async::Ready(Some(NativeEndian::read_u64(&mut buf)))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e.into()),
        }
    }
}

impl AsRawFd for EventFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.get_ref().as_raw_fd()
    }
}
