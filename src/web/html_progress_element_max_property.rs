use super::html_progress_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "max", get_max, set_max)
}

fn get_max(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, x.max).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_max(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(scope).unwrap_or(1.0);
    let v = if v > 0.0 { v } else { 1.0 };
    if let Some(x) = scope
        .get_slot_mut::<HtmlProgressElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.max = v;
        x.value = x.value.min(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
