use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "setSelectionRange",
        2,
        set_selection_range,
    )
}

fn set_selection_range(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    let length = current.value.chars().count() as u32;
    let start = a.get(0).uint32_value(scope).unwrap_or(0).min(length);
    let end = a.get(1).uint32_value(scope).unwrap_or(0).min(length);
    let start = start.min(end);
    let direction = if a.length() > 2 {
        let value = crate::webidl::value_to_string(scope, a.get(2));
        if matches!(value.as_str(), "forward" | "backward" | "none") {
            value
        } else {
            "none".to_owned()
        }
    } else {
        "none".to_owned()
    };
    update(scope, a.this(), |x| {
        x.selection_start = start;
        x.selection_end = end;
        x.selection_direction = direction;
    });
}
