pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "clients", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::get_service_object(
        scope,
        super::worker_global_scope::ServiceObjectKind::Clients,
        result,
    )
}
