use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setRangeText", 1, set_range_text)
}

fn set_range_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let replacement = crate::webidl::value_to_string(scope, a.get(0));
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    let length = current.value.chars().count() as u32;
    let (start, end) = if a.length() >= 3 {
        (
            a.get(1).uint32_value(scope).unwrap_or(0).min(length),
            a.get(2).uint32_value(scope).unwrap_or(0).min(length),
        )
    } else {
        (current.selection_start, current.selection_end)
    };
    if start > end {
        throw_range_error(scope, "The start index is greater than the end index");
        return;
    }
    let selection_mode = if a.length() >= 4 {
        crate::webidl::value_to_string(scope, a.get(3))
    } else {
        "preserve".to_owned()
    };
    let before = current
        .value
        .chars()
        .take(start as usize)
        .collect::<String>();
    let after = current.value.chars().skip(end as usize).collect::<String>();
    let replacement_length = replacement.chars().count() as u32;
    let new_value = format!("{before}{replacement}{after}");
    let replaced = end - start;
    let (selection_start, selection_end) = match selection_mode.as_str() {
        "select" => (start, start + replacement_length),
        "start" => (start, start),
        "end" => (start + replacement_length, start + replacement_length),
        _ => {
            let adjust = replacement_length as i64 - replaced as i64;
            let adjust_index = |index: u32| {
                if index <= start {
                    index
                } else if index >= end {
                    (index as i64 + adjust).max(0) as u32
                } else {
                    start + replacement_length
                }
            };
            (
                adjust_index(current.selection_start),
                adjust_index(current.selection_end),
            )
        }
    };
    update(scope, a.this(), |x| {
        x.value = new_value;
        x.value_dirty = true;
        x.selection_start = selection_start;
        x.selection_end = selection_end;
        x.selection_direction = "forward".to_owned();
    });
}
