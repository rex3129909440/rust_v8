use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "selectionDirection",
        get_selection_direction,
        set_selection_direction,
    )
}

fn get_selection_direction(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if supports_selection(&record.input_type) {
            if let Some(value) = v8::String::new(scope, &record.selection_direction) {
                r.set(value.into());
            }
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_selection_direction(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    let value = if matches!(value.as_str(), "forward" | "backward" | "none") {
        value
    } else {
        "none".to_owned()
    };
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    update(scope, a.this(), |x| x.selection_direction = value);
}
