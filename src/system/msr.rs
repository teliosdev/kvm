use error::*;
use kvm_sys as kvm;
use std::alloc::{Alloc, Global, Layout};
use std::ptr::NonNull;

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

fn generate_layout_list(count: usize) -> Layout {
    Layout::new::<kvm::MsrList>()
        .extend(Layout::array::<u32>(count).unwrap())
        .unwrap()
        .0
}

pub(super) fn alloc_list(count: usize) -> *mut kvm::MsrList {
    let layout = generate_layout_list(count);
    let pointer = unsafe { Global::default().alloc(layout).unwrap() }.cast::<kvm::MsrList>();
    pointer.as_ptr()
}

pub(super) fn condense_list(pointer: *mut kvm::MsrList, count: usize) -> Vec<MsrIndex> {
    let slice =
        unsafe { ::std::slice::from_raw_parts(&(*pointer).indicies[0], (*pointer).nmsrs as usize) };
    let result = slice.into_iter().cloned().map(MsrIndex).collect();
    unsafe {
        Global::default().dealloc(
            NonNull::new_unchecked(pointer as *mut _),
            generate_layout_list(count),
        );
    }

    result
}
