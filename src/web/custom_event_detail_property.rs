use super::custom_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "detail", get_detail)
}

fn get_detail(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(detail) = scope
        .get_slot::<CustomEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.detail.clone())
    {
        result.set(v8::Local::new(scope, &detail));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
