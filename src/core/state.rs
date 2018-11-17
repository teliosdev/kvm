use kvm_sys as kvm;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum State {
    /// The vCPU is currently running.  Only supported on x86, ARM, and arm64.
    Runnable = kvm::KVM_MP_STATE_RUNNABLE,
    /// The vCPU is an application processor which has not yet received an INIT
    /// signal.  Only supported on x86.
    Uninitialized = kvm::KVM_MP_STATE_UNINITIALIZED,
    /// The vCPU has received an INIT signal, and is now ready for a SIPI.
    /// Only supoprted on x86.
    InitReceived = kvm::KVM_MP_STATE_INIT_RECEIVED,
    /// The vCPU has executed a HLT instruction and is waiting for an interrupt.
    /// Only supported on x86
    Halted = kvm::KVM_MP_STATE_HALTED,
    /// The vCPU has just received a SIPI.  Only supported on x86.
    SipiReceived = kvm::KVM_MP_STATE_SIPI_RECEIVED,
    /// The vCPU is stopped.  Only supported on s390, ARM, and arm64.
    Stopped = kvm::KVM_MP_STATE_STOPPED,
    /// The vCPU is in a special error state.  Only supported on s390.
    CheckStop = kvm::KVM_MP_STATE_CHECK_STOP,
    /// The vCPU is operating (running or halted).  Only supported on s390.
    Operating = kvm::KVM_MP_STATE_OPERATING,
    /// The vCPU is in a special load/startup state.  Only supported on s390
    Load = kvm::KVM_MP_STATE_LOAD,
}
