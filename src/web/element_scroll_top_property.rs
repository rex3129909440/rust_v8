pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "scrollTop", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::element::record(scope, arguments.this()) {
        Some(record) => result.set(v8::Number::new(scope, record.scroll_top).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let top = arguments.get(0).number_value(scope).unwrap_or(0.0);
    super::element::set_scroll_position(scope, arguments.this(), record.scroll_left, top, false);
}
