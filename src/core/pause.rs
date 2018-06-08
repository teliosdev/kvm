use kvm_sys as sys;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    In = sys::KVM_EXIT_IO_IN,
    Out = sys::KVM_EXIT_IO_OUT,
}

impl Direction {
    pub fn reverse(&self) -> Direction {
        match self {
            Direction::In => Direction::Out,
            Direction::Out => Direction::In,
        }
    }
}

impl From<u8> for Direction {
    fn from(v: u8) -> Direction {
        match v {
            sys::KVM_EXIT_IO_IN => Direction::In,
            sys::KVM_EXIT_IO_OUT => Direction::Out,
            _ => unreachable!(),
        }
    }
}

impl Into<u8> for Direction {
    fn into(self) -> u8 {
        match self {
            Direction::In => sys::KVM_EXIT_IO_IN,
            Direction::Out => sys::KVM_EXIT_IO_OUT,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Pause {
    Unknown(u64),
    FailEntry(u64),
    Exception(u32, u32),
    Io {
        direction: Direction,
        size: u8,
        port: u16,
        count: u32,
        data_offset: u64,
    },
    Mmio {
        address: u64,
        data: [u8; 8],
        length: u32,
        is_write: bool,
    },
    SystemEvent(u32, u64),
    InternalError(u32),
    Shutdown,
    Invalid(u32),
}

impl From<sys::Run> for Pause {
    fn from(run: sys::Run) -> Pause {
        match run.exit_reason {
            sys::KVM_EXIT_UNKNOWN => Pause::Unknown(unsafe { run.exit.hw.hardware_exit_reason }),
            sys::KVM_EXIT_FAIL_ENTRY => {
                Pause::FailEntry(unsafe { run.exit.fail_entry.hardware_entry_failure_reason })
            }
            sys::KVM_EXIT_EXCEPTION => Pause::Exception(unsafe { run.exit.ex.exception }, unsafe {
                run.exit.ex.error_code
            }),
            sys::KVM_EXIT_IO => Pause::Io {
                direction: unsafe { run.exit.io.direction.into() },
                size: unsafe { run.exit.io.size },
                port: unsafe { run.exit.io.port },
                count: unsafe { run.exit.io.count },
                data_offset: unsafe { run.exit.io.data_offset },
            },
            sys::KVM_EXIT_MMIO => Pause::Mmio {
                address: unsafe { run.exit.mmio.phys_addr },
                data: unsafe { run.exit.mmio.data },
                length: unsafe { run.exit.mmio.len },
                is_write: unsafe { run.exit.mmio.is_write != 0 },
            },
            sys::KVM_EXIT_SYSTEM_EVENT => Pause::SystemEvent(
                unsafe { run.exit.system_event.kind },
                unsafe { run.exit.system_event.flags },
            ),
            sys::KVM_EXIT_INTERNAL_ERROR => {
                Pause::InternalError(unsafe { run.exit.internal.suberror })
            }
            sys::KVM_EXIT_SHUTDOWN => Pause::Shutdown,
            v => Pause::Invalid(v),
        }
    }
}

impl Into<(u32, sys::Exit)> for Pause {
    fn into(self) -> (u32, sys::Exit) {
        match self {
            Pause::Unknown(v) => (
                sys::KVM_EXIT_UNKNOWN,
                sys::Exit {
                    hw: sys::run::ExitUnknown {
                        hardware_exit_reason: v,
                    },
                },
            ),
            Pause::FailEntry(v) => (
                sys::KVM_EXIT_FAIL_ENTRY,
                sys::Exit {
                    fail_entry: sys::run::ExitFailEntry {
                        hardware_entry_failure_reason: v,
                    },
                },
            ),
            Pause::Exception(exception, error_code) => (
                sys::KVM_EXIT_EXCEPTION,
                sys::Exit {
                    ex: sys::run::ExitException {
                        exception,
                        error_code,
                    },
                },
            ),
            Pause::Io {
                direction,
                size,
                port,
                count,
                data_offset,
            } => (
                sys::KVM_EXIT_IO,
                sys::Exit {
                    io: sys::run::ExitIo {
                        direction: direction.into(),
                        size,
                        port,
                        count,
                        data_offset,
                    },
                },
            ),
            Pause::Mmio {
                address,
                data,
                length,
                is_write,
            } => (
                sys::KVM_EXIT_MMIO,
                sys::Exit {
                    mmio: sys::run::ExitMmio {
                        phys_addr: address,
                        data,
                        len: length,
                        is_write: is_write as u8,
                    },
                },
            ),
            Pause::SystemEvent(kind, flags) => (
                sys::KVM_EXIT_SYSTEM_EVENT,
                sys::Exit {
                    system_event: sys::run::ExitSystemEvent { kind, flags },
                },
            ),
            Pause::InternalError(suberror) => (
                sys::KVM_EXIT_INTERNAL_ERROR,
                sys::Exit {
                    internal: sys::run::ExitInternal {
                        suberror,
                        ndata: 0,
                        data: [0; 16],
                    },
                },
            ),
            Pause::Shutdown => (sys::KVM_EXIT_SHUTDOWN, sys::Exit { _pad: [0; 256] }),
            Pause::Invalid(v) => (v, sys::Exit { _pad: [0; 256] }),
        }
    }
}
