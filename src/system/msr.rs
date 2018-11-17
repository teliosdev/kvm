use error::*;
use kvm_sys as kvm;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// An Msr Index.
pub struct MsrIndex(pub(super) u32);

impl MsrIndex {
    /// Creates a new Msr Index value from the given value.
    pub fn new(value: u32) -> Result<MsrIndex> {
        Ok(MsrIndex(value))
    }
}

impl Into<u32> for MsrIndex {
    fn into(self) -> u32 {
        self.0
    }
}

pub(super) fn alloc_list(count: usize) -> *mut kvm::MsrList {
    use nix::libc::malloc;
    use std::mem::size_of;
    unsafe { malloc(size_of::<kvm::MsrList>() + count * size_of::<u32>()) as *mut kvm::MsrList }
}

pub(super) fn condense_list(pointer: *mut kvm::MsrList) -> Vec<MsrIndex> {
    let slice =
        unsafe { ::std::slice::from_raw_parts(&(*pointer).indicies[0], (*pointer).nmsrs as usize) };
    let result = slice.into_iter().cloned().map(MsrIndex).collect();
    unsafe {
        nix::libc::free(pointer as *mut nix::libc::c_void);
    }

    result
}
