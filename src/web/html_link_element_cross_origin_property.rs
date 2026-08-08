use super::html_link_element::*;

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
    if let Some(record) = record(scope, a.this()) {
        if let Some(cross_origin) = record.cross_origin {
            if let Some(value) = v8::String::new(scope, &cross_origin) {
                r.set(value.into());
            }
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if a.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(0)))
    };
    update(scope, a.this(), |record| record.cross_origin = value);
}
