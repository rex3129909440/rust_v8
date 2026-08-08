use super::rtc_error_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "error", get_error)
}

fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(error) = scope.get_slot::<RtcErrorEventStore>().and_then(|store| {
        store
            .records
            .get(&arguments.this().get_identity_hash().get())
    }) {
        result.set(v8::Local::new(scope, error).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
