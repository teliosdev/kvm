use super::error::*;
use nix::libc::c_void;
use nix::sys::mman;
use std::ops::{Deref, DerefMut};
use std::os::unix::io::RawFd;
use std::slice;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Slab(usize, *mut u8);

impl Slab {
    pub fn from_file(fd: RawFd, offset: isize, size: usize) -> Result<Slab> {
        let prot = mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE;
        let map = mman::MapFlags::MAP_FILE | mman::MapFlags::MAP_SHARED;
        unsafe { mman::mmap(0 as *mut c_void, size, prot, map, fd, offset as i64) }
            .chain_err(|| ErrorKind::MemoryMapError)
            .map(|pointer| Slab(size, pointer as *mut u8))
    }

    pub fn from_anon(size: usize) -> Result<Slab> {
        let prot = mman::ProtFlags::PROT_READ | mman::ProtFlags::PROT_WRITE;
        let map = mman::MapFlags::MAP_ANON | mman::MapFlags::MAP_SHARED;

        unsafe { mman::mmap(0 as *mut c_void, size, prot, map, 0, 0) }
            .chain_err(|| ErrorKind::MemoryMapError)
            .map(|pointer| Slab(size, pointer as *mut u8))
    }

    pub fn address(&self) -> u64 {
        self.1 as u64
    }

    pub fn len(&self) -> usize {
        self.0
    }

    pub fn read_bytes(&mut self, at: usize, dest: &mut [u8]) {
        unsafe {
            let src = (self.address() as usize + at) as *const u8;
            ::std::ptr::copy_nonoverlapping(src, dest.as_mut_ptr(), dest.len())
        }
    }

    pub fn write_bytes(&mut self, at: usize, value: &[u8]) {
        unsafe {
            let dest = (self.address() as usize + at) as *mut u8;
            ::std::ptr::copy_nonoverlapping(value.as_ptr(), dest, value.len())
        }
    }
}

impl Deref for Slab {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.1, self.0) }
    }
}

impl DerefMut for Slab {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.1, self.0) }
    }
}

impl<T> AsRef<T> for Slab {
    fn as_ref(&self) -> &T {
        unsafe { &*(self.address() as *const T) }
    }
}

impl<T> AsMut<T> for Slab {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.address() as *mut T) }
    }
}

impl Drop for Slab {
    fn drop(&mut self) {
        unsafe { mman::munmap(self.1 as *mut c_void, self.0) }.unwrap()
    }
}

impl Into<Arc<Mutex<Slab>>> for Slab {
    fn into(self) -> Arc<Mutex<Slab>> {
        Arc::new(Mutex::new(self))
    }
}

unsafe impl Send for Slab {}
unsafe impl Sync for Slab {}
