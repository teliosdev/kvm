error_chain!{
    errors {
        UnavailableSystemError {
            description("unable to open kvm device")
            display("unable to open kvm device")
        }

        CreateIoEventFdError {

        }

        ReadIoEventFdError {

        }

        CreateIrqFdError {}
        NotifyIrqFdError {}

        SystemApiError(req: &'static str) {
            description("an error occurred while trying to handle an api request")
            display("an error occurred while trying to handle api request `{}'", req)
        }

        MachineApiError(req: &'static str) {
            description("an error occurred while trying to handle an api request")
            display("an error occurred while trying to handle api request `{}'", req)
        }

        CoreApiError(req: &'static str) {
            description("an error occurred while trying to handle an api request")
            display("an error occurred while trying to handle api request `{}'", req)
        }

        MapCoreError {
            description("an error occurred while attempting to map the core into memory")
            display("an error occurred while attempting to map the core into memory")
        }

        MissingExtensionError(cap: ::machine::Capability) {
            description("a requested extension was missing from the system")
            display("the extension {:?} was missing from the system", cap)
        }

        InvalidVersionError(got: i32, expected: i32) {
            description("invalid KVM API version received")
            display("invalid KVM API version received; expected {}, got {}", expected, got)
        }
    }
}
