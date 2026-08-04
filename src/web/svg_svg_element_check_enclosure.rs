use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "checkEnclosure", 2, check_enclosure)
}

fn check_enclosure(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = record(scope, arguments.this()).is_some()
        && arguments.get(0).is_object()
        && arguments.get(1).is_object();
    result.set(v8::Boolean::new(scope, valid).into());
}
