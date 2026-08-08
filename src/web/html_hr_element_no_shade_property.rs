use super::html_hr_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "noShade", get_no_shade, set_no_shade)
}

fn get_no_shade(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.no_shade).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_no_shade(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<HtmlHrElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.no_shade = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
