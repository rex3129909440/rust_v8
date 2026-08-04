use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "translate", get_value, set_value)
}

fn translate_enabled<'s>(
    scope: &v8::PinScope<'s, '_>,
    mut element: v8::Local<'s, v8::Object>,
) -> bool {
    loop {
        if let Some(value) = super::element::attribute_value(scope, element, "translate") {
            if value.eq_ignore_ascii_case("no") {
                return false;
            }
            if value.is_empty() || value.eq_ignore_ascii_case("yes") {
                return true;
            }
        }
        let Some(parent) = super::node::parent(scope, element) else {
            return true;
        };
        if super::element::record(scope, parent).is_none() {
            return true;
        }
        element = parent;
    }
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
    result.set(v8::Boolean::new(scope, translate_enabled(scope, arguments.this())).into());
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
    let value = if arguments.get(0).boolean_value(scope) {
        "yes"
    } else {
        "no"
    };
    super::element::set_attribute_full(
        scope,
        arguments.this(),
        "translate".to_owned(),
        value.to_owned(),
        None,
    );
}
