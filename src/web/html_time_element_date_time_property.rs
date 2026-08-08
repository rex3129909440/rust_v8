use super::html_time_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "dateTime", get_date_time, set_date_time)
}

fn get_date_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope.get_slot::<HtmlTimeElementStore>().and_then(|store| {
        store
            .date_time
            .get(&arguments.this().get_identity_hash().get())
    }) {
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_date_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(current) = scope
        .get_slot_mut::<HtmlTimeElementStore>()
        .and_then(|store| {
            store
                .date_time
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
