use kvm_sys as kvm;

/// A builder for a region.  This is used to create a [`Region`], which
/// is then passed to the machine to set the region information.
pub struct RegionOptions<'s> {
    slot: u32,
    flags: RegionFlags,
    source: Option<&'s mut [u8]>,
    addr: u64,
}

bitflags! {
    struct RegionFlags: u32 {
        const LOG_DIRTY_PAGES = kvm::KVM_MEM_LOG_DIRTY_PAGES;
        const READ_ONLY = kvm::KVM_MEM_READONLY;
    }
}

impl<'s> RegionOptions<'s> {
    /// Create a new region builder with the given slot number.  This
    /// sets the flags to empty, i.e. neither log dirty pages nor
    /// read-only.  The source is set to be null (e.g. 0), the size is
    /// set to be 0, and the guest address is set to be 0.  In effect,
    /// this region by default does not mount anything anywhere.
    ///
    /// Bits 0-15 of the slot should be used to identify the slot.  If
    /// [`Machine::address_space_count`] returns a value greater than
    /// zero, higher bits of the slot specify the address space to
    /// modify.
    pub fn new(slot: u32) -> RegionOptions<'static> {
        RegionOptions {
            slot,
            flags: RegionFlags::empty(),
            source: None,
            addr: 0,
        }
    }

    /// Sets whether or not the virtual machine should log dirty pages.
    /// This means that whenever the guest writes to a page, the kernel
    /// will log it, which can later be retrieved through the machine
    /// interface.  This is useful for something like live migration,
    /// which copies over all of the memory while the machine is
    /// running, as it can copy the lesser-modified pages, and
    /// incremently copy over the now "dirty" pages.  This, however,
    /// does not need to be on during normal operation.
    pub fn log_dirty_pages(&mut self) -> &mut Self {
        self.flags |= RegionFlags::LOG_DIRTY_PAGES;
        self
    }

    /// Disables logging of dirty pages.  See
    /// [`RegionOptions::log_dirty_pages`] for more information.
    pub fn disable_log_dirty_pages(&mut self) -> &mut Self {
        self.flags &= !RegionFlags::LOG_DIRTY_PAGES;
        self
    }

    /// Sets whether or not the region should be read-only; i.e. that
    /// writes from the guest are not passed to the backing memory.
    /// Instead, writes are handled as an MMIO exit for the core that
    /// performed the write.
    pub fn read_only(&mut self) -> &mut Self {
        self.flags |= RegionFlags::READ_ONLY;
        self
    }

    /// Disables read-only.  See [`RegionOptions::read_only`] for more
    /// information.
    pub fn disable_read_only(&mut self) -> &mut Self {
        self.flags &= !RegionFlags::READ_ONLY;
        self
    }

    /// The pointer to the memory that should back the region.  Ideally,
    /// this might be some sort of memory map.
    ///
    /// Keep in mind there is a massive performance benefit for having
    /// the lower 21 bits of this be the same as the address, as that
    /// allows the host to optimize the use of pages for the guest.
    ///
    /// Please note that this slice *must* be valid for the lifetime of
    /// the machine, or when the region is destroyed, whichever comes
    /// first.
    pub fn source(&mut self, source: &'s mut [u8]) -> &mut Self {
        self.source = Some(source);
        self
    }

    /// This removes the source from the active region.  This turns it
    /// into a static lifetime, as it's no longer tied to anything, and
    /// returns the previous source, if it existed.
    ///
    /// Note that this does not take a reference.  This is because of
    /// the aforementioned transformation into a static lifetime.
    pub fn take(mut self) -> (RegionOptions<'static>, Option<&'s mut [u8]>) {
        let source = self.source.take();
        (unsafe { ::std::mem::transmute(self) }, source)
    }

    /// The address, within the guest, that this region should be
    /// mounted at.
    pub fn addr(&mut self, addr: u64) -> &mut Self {
        self.addr = addr;
        self
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A single region in memory for the machine.  This contains a
/// reference to the userspace memory set for the region.  It is valid
/// for this region to be null.  However, it is not valid for this
/// reference to be invalid, and so the data contained within this
/// region must be valid for at least the lifetime of the region.
/// Unfortunately, expressing such is a difficult task.
pub struct Region<'s>(u32, RegionFlags, Option<&'s mut [u8]>, u64);

impl<'s> Into<Region<'s>> for RegionOptions<'s> {
    fn into(self) -> Region<'s> {
        Region(self.slot, self.flags, self.source, self.addr)
    }
}

impl<'s> Into<RegionOptions<'s>> for Region<'s> {
    fn into(self) -> RegionOptions<'s> {
        RegionOptions {
            slot: self.0,
            flags: self.1,
            source: self.2,
            addr: self.3,
        }
    }
}

#[doc(hidden)]
impl<'s> Into<kvm::UserspaceMemoryRegion> for Region<'s> {
    fn into(mut self) -> kvm::UserspaceMemoryRegion {
        let memory_size = { self.2.as_ref().map(|v| v.len()) }.unwrap_or(0) as u64;
        let userspace_addr =
            { self.2.as_mut().map(|v| v.as_mut_ptr()) }.unwrap_or(0 as *mut _) as u64;
        kvm::UserspaceMemoryRegion {
            slot: self.0,
            flags: self.1.bits(),
            guest_phys_addr: self.3,
            memory_size,
            userspace_addr,
        }
    }
}
