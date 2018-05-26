use kvm_sys as sys;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MpState {
    /// The vCPU is currently running.  Only supported on x86, ARM, and arm64.
    Runnable,
    /// The vCPU is an application processor which has not yet received an INIT
    /// signal.  Only supported on x86.
    Uninitialized,
    /// The vCPU has received an INIT signal, and is now ready for a SIPI.
    /// Only supoprted on x86.
    InitReceived,
    /// The vCPU has executed a HLT instruction and is waiting for an interrupt.
    /// Only supported on x86
    Halted,
    /// The vCPU has just received a SIPI.  Only supported on x86.
    SipiReceived,
    /// The vCPU is stopped.  Only supported on s390, ARM, and arm64.
    Stopped,
    /// The vCPU is in a special error state.  Only supported on s390.
    CheckStop,
    /// The vCPU is operating (running or halted).  Only supported on s390.
    Operating,
    /// The vCPU is in a special load/startup state.  Only supported on s390.
    Load,
}

impl Into<u32> for MpState {
    fn into(self) -> u32 {
        match self {
            MpState::Runnable => sys::KVM_MP_STATE_RUNNABLE,
            MpState::Uninitialized => sys::KVM_MP_STATE_UNINITIALIZED,
            MpState::InitReceived => sys::KVM_MP_STATE_INIT_RECEIVED,
            MpState::Halted => sys::KVM_MP_STATE_HALTED,
            MpState::SipiReceived => sys::KVM_MP_STATE_SIPI_RECEIVED,
            MpState::Stopped => sys::KVM_MP_STATE_STOPPED,
            MpState::CheckStop => sys::KVM_MP_STATE_CHECK_STOP,
            MpState::Operating => sys::KVM_MP_STATE_OPERATING,
            MpState::Load => sys::KVM_MP_STATE_LOAD,
        }
    }
}

impl From<u32> for MpState {
    fn from(value: u32) -> MpState {
        match value {
            sys::KVM_MP_STATE_RUNNABLE => MpState::Runnable,
            sys::KVM_MP_STATE_UNINITIALIZED => MpState::Uninitialized,
            sys::KVM_MP_STATE_INIT_RECEIVED => MpState::InitReceived,
            sys::KVM_MP_STATE_HALTED => MpState::Halted,
            sys::KVM_MP_STATE_SIPI_RECEIVED => MpState::SipiReceived,
            sys::KVM_MP_STATE_STOPPED => MpState::Stopped,
            sys::KVM_MP_STATE_CHECK_STOP => MpState::CheckStop,
            sys::KVM_MP_STATE_OPERATING => MpState::Operating,
            sys::KVM_MP_STATE_LOAD => MpState::Load,
            _ => unreachable!(),
        }
    }
}
