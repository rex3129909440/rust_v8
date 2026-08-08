use super::html_data_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(current) = scope
        .get_slot::<HtmlDataElementStore>()
        .and_then(|store| store.values.get(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, current) {
            r.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(current) = scope
        .get_slot_mut::<HtmlDataElementStore>()
        .and_then(|store| store.values.get_mut(&a.this().get_identity_hash().get()))
    {
        *current = value
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
