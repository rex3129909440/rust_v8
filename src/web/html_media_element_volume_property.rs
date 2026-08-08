use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "volume", get_volume, set_volume)
}

fn get_volume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.volume).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_volume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(1.0);
    if !(0.0..=1.0).contains(&value) {
        if let Some(message) =
            v8::String::new(scope, "The volume provided is outside the range [0, 1]")
        {
            let exception = v8::Exception::range_error(scope, message);
            scope.throw_exception(exception);
        }
        return;
    }
    update(scope, arguments.this(), |record| record.volume = value);
}
