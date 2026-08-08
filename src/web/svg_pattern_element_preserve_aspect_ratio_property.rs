use super::svg_pattern_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "preserveAspectRatio",
        get_preserve_aspect_ratio,
    )
}

fn get_preserve_aspect_ratio(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.preserve_aspect_ratio, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
