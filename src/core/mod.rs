use super::error::*;
use kvm_sys as kvm;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

mod data;
mod exit;
mod state;

pub use self::data::{Data, DataMut};
pub use self::exit::{Exit, ExitMut};
pub use self::state::State;

#[derive(Debug)]
pub struct Core(pub(crate) File, *mut kvm::Run);

impl Core {
    pub(super) fn new(fd: RawFd) -> Result<Core> {
        let file = unsafe { File::from_raw_fd(fd) };
        let map = map_fd(fd)?;
        Ok(Core(file, map))
    }

    /// Returns the current state of the core.  See [`State`] for more
    /// information.
    pub fn state(&self) -> Result<State> {
        let mut mp_state = kvm::MpState { mp_state: 0 };
        unsafe {
            kvm::kvm_get_mp_state(self.as_raw_fd(), &mut mp_state)
                .chain_err(|| ErrorKind::CoreApiError("kvm_get_mp_state"))?;
            Ok(::std::mem::transmute(mp_state.mp_state))
        }
    }

    /// Sets the current state of the core.  See [`State`] for more
    /// information.
    pub fn set_state(&self, state: State) -> Result<()> {
        let state = kvm::MpState {
            mp_state: state as u32,
        };
        unsafe {
            kvm::kvm_set_mp_state(self.as_raw_fd(), &state)
                .chain_err(|| ErrorKind::CoreApiError("kvm_set_mp_state"))?;
        }
        Ok(())
    }

    /// Retrieves a read-only version of the data for the CPU.  Since
    /// CPUs cannot be sent across threads, this is safe.
    pub fn data<'c>(&'c self) -> Data<'c> {
        Data(unsafe { &*self.1 })
    }

    /// Retrieves a read-write version of the data for the CPU.  Since
    /// this requires a mutable reference to the CPU, and the CPU cannot
    /// be shared across threads, this is safe.
    pub fn data_mut<'c>(&'c mut self) -> DataMut<'c> {
        DataMut(unsafe { &mut *self.1 })
    }

    /// Runs the vCPU.
    pub fn run(&mut self) -> Result<kvm::Run> {
        unsafe { kvm::kvm_run(self.as_raw_fd()) }
            .chain_err(|| ErrorKind::CoreApiError("kvm_run"))?;
        Ok(unsafe { *self.1 })
    }

    /// Runs the vCPU, immediately exiting after running.  This allows
    /// interrupts and the like to be propagated, if needed.
    pub fn jaunt(&mut self) -> Result<kvm::Run> {
        let previous = unsafe { (*self.1).immediate_exit };
        unsafe { (*self.1).immediate_exit = 1 };
        unsafe { kvm::kvm_run(self.as_raw_fd()) }
            .chain_err(|| ErrorKind::CoreApiError("kvm_run"))?;
        unsafe { (*self.1).immediate_exit = previous };
        Ok(unsafe { *self.1 })
    }

    /// Sends an interrupt on a given line to the CPU.  This is needed
    /// to inform the CPU of events.
    pub fn interrupt(&mut self, irq: u32) -> Result<()> {
        let interrupt = kvm::Interrupt { irq };
        unsafe { kvm::kvm_interrupt(self.as_raw_fd(), &interrupt) }
            .chain_err(|| ErrorKind::CoreApiError("kvm_interrupt"))?;
        Ok(())
    }
}

impl AsRawFd for Core {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for Core {
    unsafe fn from_raw_fd(fd: RawFd) -> Core {
        Core::new(fd).unwrap()
    }
}

impl IntoRawFd for Core {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

fn map_fd(fd: RawFd) -> Result<*mut kvm::Run> {
    use nix::libc::c_void;
    use nix::sys::mman::*;
    use std::mem::size_of;

    unsafe {
        mmap(
            0 as *mut c_void,
            size_of::<kvm::Run>(),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED | MapFlags::MAP_LOCKED,
            fd,
            0,
        )
    }.map(|point| point as *mut kvm::Run)
    .chain_err(|| ErrorKind::MapCoreError)
}
