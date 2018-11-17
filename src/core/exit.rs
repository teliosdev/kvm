use kvm_sys as kvm;
pub use kvm_sys::run::*;

#[derive(Copy, Clone)]
pub enum Exit<'c> {
    Hw(&'c ExitUnknown),
    FailEntry(&'c ExitFailEntry),
    Ex(&'c ExitException),
    Io(&'c ExitIo),
    Mmio(&'c ExitMmio),
    Hypercall(&'c ExitHypercall),
    TprAccess(&'c ExitTprAccess),
    S390Sieic(&'c ExitS390Sieic),
    S390ResetFlags(u64),
    S390Ucontrol(&'c ExitS390Ucontrol),
    Dcr(&'c ExitDcr),
    Internal(&'c ExitInternal),
    Osi(&'c ExitOsi),
    PaprHcall(&'c ExitPaprHcall),
    S390Tsch(&'c ExitS390Tsch),
    Epr(&'c ExitEpr),
    SystemEvent(&'c ExitSystemEvent),
    S390Stsi(&'c ExitS390Stsi),
    Eoi(&'c ExitEoi),
}

impl<'c> Exit<'c> {
    pub fn from(reason: u32, raw: &'c kvm::Exit) -> Option<Exit<'c>> {
        match reason {
            kvm::KVM_EXIT_UNKNOWN => Some(Exit::Hw(unsafe { &raw.hw })),
            kvm::KVM_EXIT_FAIL_ENTRY => Some(Exit::FailEntry(unsafe { &raw.fail_entry })),
            kvm::KVM_EXIT_EXCEPTION => Some(Exit::Ex(unsafe { &raw.ex })),
            kvm::KVM_EXIT_IO => Some(Exit::Io(unsafe { &raw.io })),
            kvm::KVM_EXIT_MMIO => Some(Exit::Mmio(unsafe { &raw.mmio })),
            kvm::KVM_EXIT_HYPERCALL => Some(Exit::Hypercall(unsafe { &raw.hypercall })),
            kvm::KVM_EXIT_TPR_ACCESS => Some(Exit::TprAccess(unsafe { &raw.tpr_access })),
            kvm::KVM_EXIT_S390_SIEIC => Some(Exit::S390Sieic(unsafe { &raw.s390_sieic })),
            kvm::KVM_EXIT_S390_RESET => Some(Exit::S390ResetFlags(unsafe { raw.s390_reset_flags })),
            kvm::KVM_EXIT_S390_UCONTROL => Some(Exit::S390Ucontrol(unsafe { &raw.s390_ucontrol })),
            kvm::KVM_EXIT_DCR => Some(Exit::Dcr(unsafe { &raw.dcr })),
            kvm::KVM_EXIT_INTERNAL_ERROR => Some(Exit::Internal(unsafe { &raw.internal })),
            kvm::KVM_EXIT_OSI => Some(Exit::Osi(unsafe { &raw.osi })),
            kvm::KVM_EXIT_PAPR_HCALL => Some(Exit::PaprHcall(unsafe { &raw.papr_hcall })),
            kvm::KVM_EXIT_S390_TSCH => Some(Exit::S390Tsch(unsafe { &raw.s390_tsch })),
            kvm::KVM_EXIT_EPR => Some(Exit::Epr(unsafe { &raw.epr })),
            kvm::KVM_EXIT_SYSTEM_EVENT => Some(Exit::SystemEvent(unsafe { &raw.system_event })),
            kvm::KVM_EXIT_S390_STSI => Some(Exit::S390Stsi(unsafe { &raw.s390_stsi })),
            kvm::KVM_EXIT_IOAPIC_EOI => Some(Exit::Eoi(unsafe { &raw.eoi })),
            _ => None,
        }
    }

    pub fn split(&self) -> (u32, kvm::Exit) {
        match self {
            Exit::Hw(v) => (kvm::KVM_EXIT_UNKNOWN, kvm::Exit { hw: **v }),
            Exit::FailEntry(v) => (kvm::KVM_EXIT_FAIL_ENTRY, kvm::Exit { fail_entry: **v }),
            Exit::Ex(v) => (kvm::KVM_EXIT_EXCEPTION, kvm::Exit { ex: **v }),
            Exit::Io(v) => (kvm::KVM_EXIT_IO, kvm::Exit { io: **v }),
            Exit::Mmio(v) => (kvm::KVM_EXIT_MMIO, kvm::Exit { mmio: **v }),
            Exit::Hypercall(v) => (kvm::KVM_EXIT_HYPERCALL, kvm::Exit { hypercall: **v }),
            Exit::TprAccess(v) => (kvm::KVM_EXIT_TPR_ACCESS, kvm::Exit { tpr_access: **v }),
            Exit::S390Sieic(v) => (kvm::KVM_EXIT_S390_SIEIC, kvm::Exit { s390_sieic: **v }),
            Exit::S390ResetFlags(v) => (
                kvm::KVM_EXIT_S390_RESET,
                kvm::Exit {
                    s390_reset_flags: *v,
                },
            ),
            Exit::S390Ucontrol(v) => (
                kvm::KVM_EXIT_S390_UCONTROL,
                kvm::Exit { s390_ucontrol: **v },
            ),
            Exit::Dcr(v) => (kvm::KVM_EXIT_DCR, kvm::Exit { dcr: **v }),
            Exit::Internal(v) => (kvm::KVM_EXIT_INTERNAL_ERROR, kvm::Exit { internal: **v }),
            Exit::Osi(v) => (kvm::KVM_EXIT_OSI, kvm::Exit { osi: **v }),
            Exit::PaprHcall(v) => (kvm::KVM_EXIT_PAPR_HCALL, kvm::Exit { papr_hcall: **v }),
            Exit::S390Tsch(v) => (kvm::KVM_EXIT_S390_TSCH, kvm::Exit { s390_tsch: **v }),
            Exit::Epr(v) => (kvm::KVM_EXIT_EPR, kvm::Exit { epr: **v }),
            Exit::SystemEvent(v) => (kvm::KVM_EXIT_SYSTEM_EVENT, kvm::Exit { system_event: **v }),
            Exit::S390Stsi(v) => (kvm::KVM_EXIT_S390_STSI, kvm::Exit { s390_stsi: **v }),
            Exit::Eoi(v) => (kvm::KVM_EXIT_IOAPIC_EOI, kvm::Exit { eoi: **v }),
        }
    }
}

pub enum ExitMut<'c> {
    Hw(&'c mut ExitUnknown),
    FailEntry(&'c mut ExitFailEntry),
    Ex(&'c mut ExitException),
    Io(&'c mut ExitIo),
    Mmio(&'c mut ExitMmio),
    Hypercall(&'c mut ExitHypercall),
    TprAccess(&'c mut ExitTprAccess),
    S390Sieic(&'c mut ExitS390Sieic),
    S390ResetFlags(u64),
    S390Ucontrol(&'c mut ExitS390Ucontrol),
    Dcr(&'c mut ExitDcr),
    Internal(&'c mut ExitInternal),
    Osi(&'c mut ExitOsi),
    PaprHcall(&'c mut ExitPaprHcall),
    S390Tsch(&'c mut ExitS390Tsch),
    Epr(&'c mut ExitEpr),
    SystemEvent(&'c mut ExitSystemEvent),
    S390Stsi(&'c mut ExitS390Stsi),
    Eoi(&'c mut ExitEoi),
}

impl<'c> ExitMut<'c> {
    pub fn from(reason: u32, raw: &'c mut kvm::Exit) -> Option<ExitMut<'c>> {
        match reason {
            kvm::KVM_EXIT_UNKNOWN => Some(ExitMut::Hw(unsafe { &mut raw.hw })),
            kvm::KVM_EXIT_FAIL_ENTRY => Some(ExitMut::FailEntry(unsafe { &mut raw.fail_entry })),
            kvm::KVM_EXIT_EXCEPTION => Some(ExitMut::Ex(unsafe { &mut raw.ex })),
            kvm::KVM_EXIT_IO => Some(ExitMut::Io(unsafe { &mut raw.io })),
            kvm::KVM_EXIT_MMIO => Some(ExitMut::Mmio(unsafe { &mut raw.mmio })),
            kvm::KVM_EXIT_HYPERCALL => Some(ExitMut::Hypercall(unsafe { &mut raw.hypercall })),
            kvm::KVM_EXIT_TPR_ACCESS => Some(ExitMut::TprAccess(unsafe { &mut raw.tpr_access })),
            kvm::KVM_EXIT_S390_SIEIC => Some(ExitMut::S390Sieic(unsafe { &mut raw.s390_sieic })),
            kvm::KVM_EXIT_S390_RESET => {
                Some(ExitMut::S390ResetFlags(unsafe { raw.s390_reset_flags }))
            }
            kvm::KVM_EXIT_S390_UCONTROL => {
                Some(ExitMut::S390Ucontrol(unsafe { &mut raw.s390_ucontrol }))
            }
            kvm::KVM_EXIT_DCR => Some(ExitMut::Dcr(unsafe { &mut raw.dcr })),
            kvm::KVM_EXIT_INTERNAL_ERROR => Some(ExitMut::Internal(unsafe { &mut raw.internal })),
            kvm::KVM_EXIT_OSI => Some(ExitMut::Osi(unsafe { &mut raw.osi })),
            kvm::KVM_EXIT_PAPR_HCALL => Some(ExitMut::PaprHcall(unsafe { &mut raw.papr_hcall })),
            kvm::KVM_EXIT_S390_TSCH => Some(ExitMut::S390Tsch(unsafe { &mut raw.s390_tsch })),
            kvm::KVM_EXIT_EPR => Some(ExitMut::Epr(unsafe { &mut raw.epr })),
            kvm::KVM_EXIT_SYSTEM_EVENT => {
                Some(ExitMut::SystemEvent(unsafe { &mut raw.system_event }))
            }
            kvm::KVM_EXIT_S390_STSI => Some(ExitMut::S390Stsi(unsafe { &mut raw.s390_stsi })),
            kvm::KVM_EXIT_IOAPIC_EOI => Some(ExitMut::Eoi(unsafe { &mut raw.eoi })),
            _ => None,
        }
    }

    // pub fn as_ref(&self) -> Exit<'c> {
    //     match self {
    //         ExitMut::Hw(v) => Exit::Hw(&*v),
    //         ExitMut::FailEntry(v) => Exit::FailEntry(&*v),
    //         ExitMut::Ex(v) => Exit::Ex(&*v),
    //         ExitMut::Io(v) => Exit::Io(&*v),
    //         ExitMut::Mmio(v) => Exit::Mmio(&*v),
    //         ExitMut::Hypercall(v) => Exit::Hypercall(&*v),
    //         ExitMut::TprAccess(v) => Exit::TprAccess(&*v),
    //         ExitMut::S390Sieic(v) => Exit::S390Sieic(&*v),
    //         ExitMut::S390ResetFlags(v) => Exit::S390ResetFlags(*v),
    //         ExitMut::S390Ucontrol(v) => Exit::S390Ucontrol(&*v),
    //         ExitMut::Dcr(v) => Exit::Dcr(&*v),
    //         ExitMut::Internal(v) => Exit::Internal(&*v),
    //         ExitMut::Osi(v) => Exit::Osi(&*v),
    //         ExitMut::PaprHcall(v) => Exit::PaprHcall(&*v),
    //         ExitMut::S390Tsch(v) => Exit::S390Tsch(&*v),
    //         ExitMut::Epr(v) => Exit::Epr(&*v),
    //         ExitMut::SystemEvent(v) => Exit::SystemEvent(&*v),
    //         ExitMut::S390Stsi(v) => Exit::S390Stsi(&*v),
    //         ExitMut::Eoi(v) => Exit::Eoi(&*v),
    //     }
    // }

    pub fn split(&self) -> (u32, kvm::Exit) {
        let result: Exit<'_> = self.into();
        result.split()
    }
}

impl<'m, 'c> Into<Exit<'m>> for &'m ExitMut<'c> {
    fn into(self) -> Exit<'m> {
        match self {
            ExitMut::Hw(v) => Exit::Hw(&*v),
            ExitMut::FailEntry(v) => Exit::FailEntry(&*v),
            ExitMut::Ex(v) => Exit::Ex(&*v),
            ExitMut::Io(v) => Exit::Io(&*v),
            ExitMut::Mmio(v) => Exit::Mmio(&*v),
            ExitMut::Hypercall(v) => Exit::Hypercall(&*v),
            ExitMut::TprAccess(v) => Exit::TprAccess(&*v),
            ExitMut::S390Sieic(v) => Exit::S390Sieic(&*v),
            ExitMut::S390ResetFlags(v) => Exit::S390ResetFlags(*v),
            ExitMut::S390Ucontrol(v) => Exit::S390Ucontrol(&*v),
            ExitMut::Dcr(v) => Exit::Dcr(&*v),
            ExitMut::Internal(v) => Exit::Internal(&*v),
            ExitMut::Osi(v) => Exit::Osi(&*v),
            ExitMut::PaprHcall(v) => Exit::PaprHcall(&*v),
            ExitMut::S390Tsch(v) => Exit::S390Tsch(&*v),
            ExitMut::Epr(v) => Exit::Epr(&*v),
            ExitMut::SystemEvent(v) => Exit::SystemEvent(&*v),
            ExitMut::S390Stsi(v) => Exit::S390Stsi(&*v),
            ExitMut::Eoi(v) => Exit::Eoi(&*v),
        }
    }
}
