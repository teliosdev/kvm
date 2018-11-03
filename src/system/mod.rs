use super::error::*;
use super::machine::{Machine, MachineKind};
use kvm_sys as kvm;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

mod msr;

pub use self::msr::MsrIndex;

#[derive(Debug)]
/// A KVM System.  This represents the host machine, most likely.
/// From this, we can create "machines."  For the purposes of KVM,
/// this is a virtual machine.  Those virtual machines can then have
/// cores themselves.
///
/// This is backed by a file descriptor from the operating system.
///
/// # Safety
/// This is not thread-safe.  If multiple threads *must* use this,
/// consider putting it behind a mutex.
pub struct System(File);

impl System {
    /// Creates a new system from the default KVM file.  Since the
    /// device name doesn't change, at all, the only reason this
    /// should fail is for Linux versions that don't support KVM.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), Box<Error>> {
    /// let system = System::new()?;
    /// // do something with system...
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// This errors if, and only if, opening the file fails.  This
    /// typically means that the operating system does not support
    /// KVM.  This will return a [`ErrorKind::UnavailableSystemError`].
    pub fn new() -> Result<System> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .map(System)
            .chain_err(|| ErrorKind::UnavailableSystemError)
    }

    /// Retrieves the KVM API version.  This should only return 12,
    /// as of the writing of this code.  To actually check the version
    /// number as well (likely the behavior you want), use
    /// [`System::verify_api_version`].
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), Box<Error>> {
    /// let system = System::new()?;
    /// let version = system.api_version()?;
    /// assert_eq!(version, 12);
    /// #    Ok(())
    /// # }
    /// ```
    /// # Errors
    /// This will fail if the API call fails.  The API call should only
    /// fail if the file descriptor backing the system is invalid,
    /// which should not be the case if [`System::new`] was used.
    pub fn api_version(&self) -> Result<i32> {
        unsafe { kvm::kvm_get_api_version(self.as_raw_fd()) }
            .chain_err(|| ErrorKind::SystemApiError("kvm_get_api_version"))
    }

    /// Verifies that the API version is the one that is compatible
    /// with this library.  This should be the preferred function over
    /// [`System::api_version`].
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<Error>> {
    /// let system = System::new()?;
    /// system.verify_api_version()?;
    /// #     Ok(())
    /// #}
    /// ```
    ///
    /// # Errors
    /// This will fail if either the API call fails, or if the API
    /// version is not the one we expected.
    pub fn verify_api_version(&self) -> Result<()> {
        self.api_version().and_then(|value| {
            if value != 12 {
                Err(ErrorKind::InvalidVersionError(value, 12).into())
            } else {
                Ok(())
            }
        })
    }

    /// Creates a virtual machine from this system.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), Box<Error>> {
    /// # use super::machine::MachineKind;
    /// let system = System::new()?;
    /// system.verify_api_version()?;
    /// let machine = system.create_machine(MachineKind::Default)?;
    /// // do something with the new machine.
    /// #     Ok(())
    /// # }
    /// # Errors
    /// This will error if the API call fails.
    pub fn create_machine(&self, kind: MachineKind) -> Result<Machine> {
        unsafe { kvm::kvm_create_vm(self.as_raw_fd(), kind as i32) }
            .map(|v| unsafe { Machine::from_raw_fd(v) })
            .chain_err(|| ErrorKind::SystemApiError("kvm_create_vm"))
    }

    pub fn msr_index_list(&self) -> Result<Vec<MsrIndex>> {
        // First, we have to figure out how many indicies there are.
        // We create a blank list.
        let mut list = kvm::MsrList {
            nmsrs: 0,
            indicies: [],
        };
        unsafe { kvm::kvm_get_msr_index_list(self.as_raw_fd(), &mut list as *mut _) }
            .chain_err(|| ErrorKind::SystemApiError("kvm_get_msr_index_list"))?;
        let count = list.nmsrs as usize;
        // Create an allocation of an msrlist for the API.
        let pointer = self::msr::alloc_list(count);

        unsafe { kvm::kvm_get_msr_index_list(self.as_raw_fd(), pointer) }
            .chain_err(|| ErrorKind::SystemApiError("kvm_get_msr_index_list"))?;

        Ok(self::msr::condense_list(pointer, count))
    }

    pub fn msr_feature_index_list(&self) -> Result<Vec<MsrIndex>> {
        let mut list = kvm::MsrList {
            nmsrs: 0,
            indicies: [],
        };
        unsafe { kvm::kvm_get_msr_feature_index_list(self.as_raw_fd(), &mut list as *mut _) }
            .chain_err(|| ErrorKind::SystemApiError("kvm_get_msr_feature_index_list"))?;
        let count = list.nmsrs as usize;
        let pointer = self::msr::alloc_list(count);
        unsafe { kvm::kvm_get_msr_feature_index_list(self.as_raw_fd(), pointer) }
            .chain_err(|| ErrorKind::SystemApiError("kvm_get_msr_feature_index_list"))?;
        Ok(self::msr::condense_list(pointer, count))
    }

    /// Returns the size required for the mmap of the vCPU file
    /// descriptor, in bytes.  This is needed for processing the
    /// structure located at that address.
    ///
    /// # Example
    /// ```rust
    /// # fn main() -> Result<(), Box<Error>> {
    /// # use super::machine::MachineKind;
    /// use memmap::MmapOptions;
    /// let system = System::new()?;
    /// system.verify_api_version()?;
    /// let machine = system.create_machine(MachineKind::Default)?;
    /// let core = machine.create_core()?;
    /// let mmap = MmapOption::new()
    ///     .len(system.core_mmap_size()?)
    ///     .map_mut(core.as_ref())?;
    /// // do things with mmap
    /// #     Ok(())
    /// # }
    pub fn core_mmap_size(&self) -> Result<usize> {
        unsafe { kvm::kvm_get_vcpu_mmap_size(self.as_raw_fd()) }
            .chain_err(|| ErrorKind::SystemApiError("kvm_get_vcpu_mmap_size"))
            .map(|v| v as usize)
    }
}

impl AsRawFd for System {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for System {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for System {
    /// This is, and should be, unsafe.  This does not check that the
    /// given file descriptor is a valid system file descriptor.  As
    /// such, if it's later used to create a virtual machine, or any
    /// other activities, it's undefined behavior.
    unsafe fn from_raw_fd(fd: RawFd) -> System {
        System(File::from_raw_fd(fd))
    }
}

impl AsRef<File> for System {
    fn as_ref(&self) -> &File {
        &self.0
    }
}

impl !Sync for System {}
