use super::{Exit, ExitMut};
use kvm_sys as kvm;

#[derive(Copy, Clone)]
pub struct Data<'c>(pub(super) &'c kvm::Run);

impl<'c> Data<'c> {
    pub fn exit_reason(&self) -> u32 {
        self.0.exit_reason
    }

    pub fn cr8(&self) -> u64 {
        self.0.cr8
    }

    pub fn raw_exit(&self) -> &kvm::Exit {
        &self.0.exit
    }

    pub fn exit(&self) -> Option<Exit<'c>> {
        Exit::from(self.exit_reason(), &self.0.exit)
    }
}

impl<'c> AsRef<kvm::Run> for Data<'c> {
    fn as_ref(&self) -> &kvm::Run {
        self.0
    }
}

pub struct DataMut<'c>(pub(super) &'c mut kvm::Run);

impl<'c> DataMut<'c> {
    pub fn exit_reason(&self) -> u32 {
        self.0.exit_reason
    }

    pub fn set_exit_reason(&mut self, reason: u32) {
        self.0.exit_reason = reason;
    }

    pub fn cr8(&self) -> u64 {
        self.0.cr8
    }

    pub fn set_cr8(&mut self, value: u64) {
        self.0.cr8 = value;
    }

    pub fn raw_exit(&self) -> &kvm::Exit {
        &self.0.exit
    }

    pub fn set_raw_exit(&mut self, exit: kvm::Exit) {
        self.0.exit = exit;
    }

    pub fn exit(&'c mut self) -> Option<ExitMut<'c>> {
        ExitMut::from(self.exit_reason(), &mut self.0.exit)
    }

    pub fn set_exit<'m>(&mut self, exit: impl Into<Exit<'m>>) {
        let exit = exit.into();
        let (reason, raw) = exit.split();
        self.set_exit_reason(reason);
        self.set_raw_exit(raw);
    }
}

impl<'c> AsRef<kvm::Run> for DataMut<'c> {
    fn as_ref(&self) -> &kvm::Run {
        self.0
    }
}

impl<'c> AsMut<kvm::Run> for DataMut<'c> {
    fn as_mut(&mut self) -> &mut kvm::Run {
        self.0
    }
}
