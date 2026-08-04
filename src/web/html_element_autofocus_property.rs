use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "autofocus", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let present = super::element::attribute_value(scope, arguments.this(), "autofocus").is_some();
    result.set(v8::Boolean::new(scope, present).into());
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.get(0).boolean_value(scope) {
        super::element::set_attribute_full(
            scope,
            arguments.this(),
            "autofocus".to_owned(),
            String::new(),
            None,
        );
    } else {
        super::element::remove_attribute_full(scope, arguments.this(), None, "autofocus");
    }
}
