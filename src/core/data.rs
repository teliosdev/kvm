use kvm_sys as kvm;

#[derive(Copy, Clone)]
pub struct Data<'c>(pub(super) &'c kvm::Run);

impl<'c> AsRef<kvm::Run> for Data<'c> {
    fn as_ref(&self) -> &kvm::Run {
        self.0
    }
}

pub struct DataMut<'c>(pub(super) &'c mut kvm::Run);

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
