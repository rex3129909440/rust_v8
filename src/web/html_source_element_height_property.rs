use super::html_source_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "height", get_height, set_height)
}

fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, x.height).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(x) = scope
        .get_slot_mut::<HtmlSourceElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.height = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
