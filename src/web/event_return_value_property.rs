use super::event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "returnValue",
        get_return_value,
        set_return_value,
    )
}

fn get_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_bool(scope, arguments, result, |record| !record.default_prevented);
}

fn set_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !arguments.get(0).boolean_value(scope) {
        update(scope, arguments.this(), |record| {
            if record.cancelable && !record.in_passive_listener {
                record.default_prevented = true;
            }
        });
    }
}
