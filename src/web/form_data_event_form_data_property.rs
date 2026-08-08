use super::form_data_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "formData", get_form_data)
}

fn get_form_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<FormDataEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
