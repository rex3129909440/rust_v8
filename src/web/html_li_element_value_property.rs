use super::html_li_element::*;

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
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Integer::new(scope, record.value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).int32_value(scope).unwrap_or(0);
    if let Some(record) = scope
        .get_slot_mut::<HtmlLiElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.value = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
