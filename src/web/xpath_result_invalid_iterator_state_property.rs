use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "invalidIteratorState",
        get_invalid_iterator_state,
    )
}

fn get_invalid_iterator_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require_record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, false).into());
    }
}
