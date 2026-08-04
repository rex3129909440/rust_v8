use super::gpu_uncaptured_error_event::*;

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
    let error = scope
        .get_slot::<GpuUncapturedErrorEventStore>()
        .and_then(|store| {
            store
                .errors
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(error) = error {
        result.set(v8::Local::new(scope, &error).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
