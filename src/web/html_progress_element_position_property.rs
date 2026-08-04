use super::html_progress_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "position", get_position)
}

fn get_position(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, if x.has_value { x.value / x.max } else { -1.0 }).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
