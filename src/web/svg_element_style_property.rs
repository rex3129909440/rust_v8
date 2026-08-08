use super::svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "style", get_style, set_style)
}

pub(crate) fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.style, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(style) = record(scope, arguments.this()).map(|record| record.style) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let (Some(key), Some(value)) = (
        v8::String::new(scope, "cssText"),
        v8::String::new(scope, &source),
    ) {
        let object = v8::Local::new(scope, &style);
        let _ = object.set(scope, key.into(), value.into());
    }
}
