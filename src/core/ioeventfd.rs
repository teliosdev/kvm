use super::super::eventfd::EventFd;
use super::IoAddress;
use error::*;
use std::io;
use std::ops::Deref;
use std::ops::DerefMut;
use std::result::Result as StdResult;
use tokio::io as tio;
use tokio::prelude::*;

pub struct IoEventFd {
    eventfd: EventFd,
    pub(super) addr: IoAddress,
    pub(super) len: u32,
}

// impl !Sync for IoEventFd {}

impl IoEventFd {
    pub(super) fn new(addr: IoAddress, len: u32) -> Result<IoEventFd> {
        Ok(IoEventFd {
            addr,
            len,
            eventfd: EventFd::new()?,
        })
    }
}

impl Deref for IoEventFd {
    type Target = EventFd;
    fn deref(&self) -> &EventFd {
        &self.eventfd
    }
}

impl DerefMut for IoEventFd {
    fn deref_mut(&mut self) -> &mut EventFd {
        &mut self.eventfd
    }
}

impl io::Read for IoEventFd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.eventfd.read(buf)
    }
}

impl io::Write for IoEventFd {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.eventfd.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.eventfd.flush()
    }
}

impl tio::AsyncRead for IoEventFd {}
impl tio::AsyncWrite for IoEventFd {
    fn shutdown(&mut self) -> StdResult<Async<()>, tio::Error> {
        self.eventfd.shutdown()
    }
}

impl stream::Stream for IoEventFd {
    type Item = u64;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<u64>>> {
        self.eventfd.poll()
    }
}
