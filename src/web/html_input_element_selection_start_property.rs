use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "selectionStart",
        get_selection_start,
        set_selection_start,
    )
}

fn get_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if supports_selection(&record.input_type) {
            r.set(v8::Integer::new_from_unsigned(scope, record.selection_start).into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).uint32_value(scope).unwrap_or(0);
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    update(scope, a.this(), |x| {
        let limit = x.value.chars().count() as u32;
        x.selection_start = value.min(limit);
        if x.selection_end < x.selection_start {
            x.selection_end = x.selection_start;
        }
    });
}
