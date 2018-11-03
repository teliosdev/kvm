use super::core::Core;
use super::error::*;
use kvm_sys as kvm;
use std::fs::File;
use std::num::NonZeroU32;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

mod ioeventfd;
mod region;
pub use self::ioeventfd::{IoEventFd, IoEventFdFlag};
pub use self::region::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u32)]
/// The IRQ level.  This does not have to deal with the implementation
/// details of the hardware, as some hardware is active-high or
/// active-low.  Here, `Active` is always active, and `Inactive` is
/// always inactive.
pub enum IrqLevel {
    Inactive = 0,
    Active = 1,
}

bitflags! {
    /// The clock flags for setting and retrieval of the CPU clock.
    pub struct ClockFlag: u32 {
        /// Indicates that the clock must be consistant across all cores
        /// when setting and retrieving the clock.
        const STABLE = kvm::KVM_CLOCK_TSC_STABLE;
    }
}

bitflags! {
    /// The flats for the PIT device.
    pub struct PitFlag: u32 {
        const SPEAKER_DUMMY = kvm::KVM_PIT_SPEAKER_DUMMY;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
/// Capability information.  This is used to ensure, check, or enable
/// capabilities on the machine.
pub enum Capability {
    NumberCores = kvm::KVM_CAP_NR_VCPUS,
    NumberMemorySlots = kvm::KVM_CAP_NR_MEMSLOTS,
    MaxCores = kvm::KVM_CAP_MAX_VCPUS,
    MaxCoreId = kvm::KVM_CAP_MAX_VCPU_ID,
    MultiAddressSpace = kvm::KVM_CAP_MULTI_ADDRESS_SPACE,
    IrqChip = kvm::KVM_CAP_IRQCHIP,
    SyncMmu = kvm::KVM_CAP_SYNC_MMU,
    SetTssAddress = kvm::KVM_CAP_SET_TSS_ADDR,
    SetIdentityMapAddress = kvm::KVM_CAP_SET_IDENTITY_MAP_ADDR,
    IoEventFd = kvm::KVM_CAP_IOEVENTFD,
    IoEventFdAnyLength = kvm::KVM_CAP_IOEVENTFD_ANY_LENGTH,
    IoEventFdNoLength = kvm::KVM_CAP_IOEVENTFD_NO_LENGTH,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
/// The virtual machine type to create.  For now, there is only one
/// value, the default.
pub enum MachineKind {
    Default = 0,
}

#[derive(Debug)]
/// A virtualized machine.  This contains and manages information
/// relating to a single virtualized instance, including the cores that
/// are included with that instance.
///
/// # Safety
/// This is not thread-safe.  If you must interact with it across
/// threads, consider using a mutex.
pub struct Machine(pub(crate) File);

impl Machine {
    /// Returns information about a specified extension/capability.
    /// The significance of the return value is dependent on the
    /// capability being requested; however, for most, a zero value
    /// indicates an absense of that capability, and a one value
    /// indicates a presence of that capability.
    pub fn extension(&self, cap: Capability) -> Result<i32> {
        unsafe { kvm::kvm_check_extension(self.as_raw_fd(), cap as i32) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_check_extension"))
    }

    /// Requires the existance of an extension on the host system.  This
    /// is similar to [`Machine::extension`], except this requires that
    /// the value be non-zero.  If the value is non-zero, it is returned.
    /// Otherwise, it is treated as an error, and
    /// [`ErrorKind::MissingExtensionError`] is returned.
    pub fn assert_extension(&self, cap: Capability) -> Result<NonZeroU32> {
        self.extension(cap)
            .chain_err(|| ErrorKind::MachineApiError("kvm_check_extension"))
            .map(|v| NonZeroU32::new(v as u32))
            .and_then(|value| {
                value
                    .map(Ok)
                    .unwrap_or_else(|| Err(ErrorKind::MissingExtensionError(cap).into()))
            })
    }

    /// Determines the max number of cores available for this machine.
    /// This determines the _absolute_ maximum number of cores; the
    /// internal API has a concept of a "recommended" number of cores,
    /// as well.  Exceeding this value in number of cores will result
    /// in an error.
    pub fn max_cores(&self) -> Result<i32> {
        self.extension(Capability::MaxCores).and_then(|value| {
            if value == 0 {
                self.extension(Capability::NumberCores)
            } else {
                Ok(value)
            }
        })
    }

    /// Determines the maximum core ID for this machine.  This can be
    /// different from the maximum number of cores.  Exceeding this
    /// value in the ID for the cores will result in an error.
    pub fn max_core_id(&self) -> Result<i32> {
        self.extension(Capability::MaxCoreId).and_then(|value| {
            if value == 0 {
                self.max_cores()
            } else {
                Ok(value)
            }
        })
    }

    /// The maximum number of slots for regions.  Values graeter than
    /// this will be rejected.
    pub fn max_region_slots(&self) -> Result<i32> {
        self.extension(Capability::NumberMemorySlots)
    }

    /// The number of address spaces supported by the Core.  If this is
    /// zero, then no address spaces are supported.
    pub fn address_space_count(&self) -> Result<i32> {
        self.extension(Capability::MultiAddressSpace)
    }

    /// Creates a single core on the machine with the given ID.  This
    /// core is a "vCPU" in KVM terminology.  Note that errors may arise
    /// for using the same id for multiple cores, exceeding the max
    /// core count, or exceeding the max core ID.
    pub fn create_core(&self, id: i32) -> Result<Core> {
        unsafe { kvm::kvm_create_vcpu(self.as_raw_fd(), id) }
            .map(|v| unsafe { Core::from_raw_fd(v) })
            .chain_err(|| ErrorKind::MachineApiError("kvm_create_vcpu"))
    }

    /// Retrieves the dirty log for the given slot.  The size here is
    /// at least the size of the memory slot registered with the
    /// machine.  This then returns a vector of numbers.  Each bit in
    /// the number represents an individual page.  If that bit is true,
    /// then the page is dirty.  If it's false, it's clean.
    pub fn dirty_log(&self, slot: u32, size: usize) -> Result<Vec<u64>> {
        // We devide the given size by the page size to get the number
        // of pages.  However, we need to round up.  So we add just
        // enough to make sure we increase the size if it's greater than
        // a page boundry, divide by the page size.
        // 10 bytes = 1 page, 4096 bytes = 1 page, 4100 bytes = 2 pages,
        // 8191 bytes = 2 pages, 8192 bytes = 2 pages, 8193 bytes = 3
        // pages, and so on.
        // Right-shifting by 12 bits is the same as dividing by 4096.
        let pages = (size + (4096 - 1)) >> 12;
        // the page count is 64-bit aligned...? But we're also packing
        // them into longs.
        let pages = ((pages + (64 - 1)) & !64) / 8;
        let mut vec = vec![0u64; pages];
        let pointer = vec.as_mut_ptr();
        let value = kvm::DirtyLog {
            slot,
            _pad: 0,
            value: kvm::DirtyLogValue {
                dirty_bitmap: pointer,
            },
        };

        let _ = unsafe { kvm::kvm_get_dirty_log(self.as_raw_fd(), &value as *const _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_get_dirty_log"))?;

        Ok(vec)
    }

    /// Creates a virtual IoApic, a virtual Pic, and causes all future
    /// cores to be created with Apics.  This is likely desirable
    /// behavior, unless you wish to implement the IRQs.  This only
    /// supports x86 platforms.
    pub fn create_irqchip(&self) -> Result<()> {
        self.assert_extension(Capability::IrqChip).and_then(|_| {
            unsafe { kvm::kvm_create_irqchip(self.as_raw_fd()) }
                .chain_err(|| ErrorKind::MachineApiError("kvm_create_irqchip"))?;
            Ok(())
        })
    }

    /// Sets the level of the given IRQ line, returning the status of
    /// that line.  Note that edge-triggered IRQs will require first
    /// setting it active, and then setting it inactive.
    ///
    /// IRQ values 0-15 go to the virtual PIC; 0-23 go to the virtual
    /// IOAPIC.
    ///
    /// This returns the status of the trigger.  If the value is less
    /// than 0, that means that the trigger was not delievered, either
    /// because it was masked or for some other reason.  If it is 0,
    /// then it was coalesced (because a previous IRQ was pending).
    /// For values greater than 0, it indicates the number of cores that
    /// the trigger was delivered to.
    pub fn set_irq_level(&self, irq: u32, level: IrqLevel) -> Result<u32> {
        let mut irqlevel = kvm::IrqLevel {
            irq,
            level: level as u32,
        };
        unsafe { kvm::kvm_irq_line_status(self.as_raw_fd(), &mut irqlevel as *mut _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_irq_line_status"))
            .map(|_| irqlevel.irq)
    }

    /// Retrieves the clock of the machine.  The flag here can specify
    /// how the clock should be retrieved.  Right now, the only flag
    /// available is the [`ClockFlag::STABLE`] flag, which denotes that
    /// the clock result should be consistent across all cores.  If this
    /// is not set, then the clock may not be consistent.
    pub fn clock(&self, flag: ClockFlag) -> Result<u64> {
        let mut clock = kvm::ClockData {
            clock: 0,
            flags: flag.bits(),
            _pad: [0; 9],
        };

        unsafe { kvm::kvm_get_clock(self.as_raw_fd(), &mut clock as *mut _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_get_clock"))
            .map(|_| clock.clock)
    }

    /// Sets the clock to the given value.  The flag here can specify
    /// how the clock should be set.  Right now, the only flag available
    /// is the [`ClockFlag::STABLE`] flag, which denotes that the clock
    /// set should be consistent across all cores.
    pub fn set_clock(&self, clock: u64, flag: ClockFlag) -> Result<()> {
        let clock = kvm::ClockData {
            clock,
            flags: flag.bits(),
            _pad: [0; 9],
        };

        unsafe { kvm::kvm_set_clock(self.as_raw_fd(), &clock as *const _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_set_clock"))
            .map(|_| ())
    }

    /// Sets a memory region for the machine.  If a region is provided
    /// with the same slot as an already existing region, that region
    /// will be updated.  Regions that overlap will be prioritised based
    /// on the higher slot number.  See [`Region`] for more information.
    pub fn set_region<'s>(&self, region: impl Into<Region<'s>>) -> Result<()> {
        let region: Region = region.into();
        let umr: kvm::UserspaceMemoryRegion = region.into();

        unsafe { kvm::kvm_set_user_memory_region(self.as_raw_fd(), &umr as *const _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_set_user_memory_region"))
            .map(|_| ())
    }

    /// This sets a region in memory that must be at least three pages
    /// long (4096 bytes * 3), within the first 4GB (<2^32-1) of memory.
    /// This *must not* conflict with any existing memory slot or
    /// MMIO address.  The guest _may not_ access this memory.
    ///
    /// This is _required_ on Intel-based machines, due to a quirk in
    /// the implementation detail.  A good choice for this may be
    /// `0xfffbd000`.
    pub fn set_tss_address(&self, address: u32) -> Result<()> {
        self.assert_extension(Capability::SetTssAddress)
            .and_then(|_| {
                unsafe { kvm::kvm_set_tss_addr(self.as_raw_fd(), address) }
                    .chain_err(|| ErrorKind::MachineApiError("kvm_set_tss_addr"))
                    .map(|_| ())
            })
    }

    /// This sets a region in memory that must be at least one page
    /// long (4096 bytes), and within the first 4GB (<2^32-1) of memory.
    /// This *must not* conflict with any existing memory slot or MMIO
    /// address.  The guest _may not_ access this memory.  This *must*
    /// be called before any cores are created, otherwise this will
    /// error.
    ///
    /// This is _required_ on Intel-based machines, due to a quirk in
    /// the implementation detail.  A good choice for this may be
    /// `0xfffbc000`.
    pub fn set_identity_address(&self, address: u64) -> Result<()> {
        self.assert_extension(Capability::SetIdentityMapAddress)
            .and_then(|_| {
                unsafe { kvm::kvm_set_identity_map_addr(self.as_raw_fd(), &address as *const _) }
                    .chain_err(|| ErrorKind::MachineApiError("kvm_set_identity_map_addr"))
                    .map(|_| ())
            })
    }

    /// Creates a Programmable Interrupt Timer used by the machine.
    /// This is powered by the kernel itself.  This operation is only
    /// valid *after* we've already created an IRQ chip.
    pub fn create_pit(&self, flags: PitFlag) -> Result<()> {
        let config = kvm::PitConfig {
            flags: flags.bits(),
            _pad: [0; 15],
        };

        unsafe { kvm::kvm_create_pit2(self.as_raw_fd(), &config as *const _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_create_pit2"))
            .map(|_| ())
    }

    pub fn create_ioeventfd<'m>(
        &'m self,
        address: u64,
        length: u32,
        data: u64,
        flags: IoEventFdFlag,
    ) -> Result<IoEventFd<'m>> {
        let eventfd = IoEventFd::build()?;

        self.ioeventfd_mod(address, length, data, flags, eventfd.as_raw_fd())
            .map(|_| IoEventFd {
                machine: self,
                file: eventfd,
                address,
                length,
                data,
                flags,
            })
    }

    pub(crate) fn ioeventfd_mod(
        &self,
        addr: u64,
        len: u32,
        datamatch: u64,
        flags: IoEventFdFlag,
        fd: RawFd,
    ) -> Result<()> {
        let ioeventfd = kvm::IoEventFd {
            addr,
            len,
            datamatch,
            fd,
            flags: flags.bits(),
            _pad: [0; 36],
        };

        unsafe { kvm::kvm_ioeventfd(self.as_raw_fd(), &ioeventfd as *const _) }
            .chain_err(|| ErrorKind::MachineApiError("kvm_ioeventfd"))
            .map(|_| ())
    }
}

impl AsRawFd for Machine {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for Machine {
    unsafe fn from_raw_fd(fd: RawFd) -> Machine {
        Machine(File::from_raw_fd(fd))
    }
}

impl IntoRawFd for Machine {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl !Sync for Machine {}
