use super::html_script_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "crossOrigin",
        get_cross_origin,
        set_cross_origin,
    )
}

fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        if let Some(v) = x.cross_origin.and_then(|v| v8::String::new(scope, &v)) {
            r.set(v.into())
        } else {
            r.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = if a.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(0)))
    };
    if let Some(x) = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.cross_origin = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
