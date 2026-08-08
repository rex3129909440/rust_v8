use super::svg_marker_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setOrientToAuto", 0, set_orient_to_auto)
}

fn set_orient_to_auto(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let orient_type = v8::Local::new(scope, &record.orient_type);
    super::svg_animated_enumeration::set(scope, orient_type, ORIENT_AUTO as u32);
}
