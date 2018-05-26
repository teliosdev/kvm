use super::capability::CapabilityKind;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        KvmSystemOpenError {
            description("could not open /dev/kvm")
            display("could not open /dev/kvm")
        }

        KvmSystemOperationError(operation: &'static str) {
            description("could not perform an operation")
            display("could not perform the operation `{}'", operation)
        }

        KvmMachineOperationError(operation: &'static str) {
            description("could not perform an operation")
            display("could not perform the operation `{}'", operation)
        }

        KvmCoreOperationError(operation: &'static str) {
            description("could not perform an operation")
            display("could not perform the operation `{}'", operation)
        }

        KvmCapabilityError(operation: &'static str) {
            description("could not check a capability")
            display("could not perform the operatoin `{}'", operation)
        }

        KvmCapabilityFailError(cap: CapabilityKind) {
            description("could not detect a given capability")
            display("could not find the capability {:?}", cap)
        }

        UnsupportedOsError {
            description("attempted to run on an unsupported OS")
            display("attempted to run on an unsupported OS")
        }

        MemoryMapError {
            description("managing a memory map failed")
            display("managing a memory map failed")
        }

        MemoryAllocationError

        TokioError
    }
}
