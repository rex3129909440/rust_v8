use super::html_text_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setRangeText", 1, set_range_text)
}

fn set_range_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let replacement = crate::webidl::value_to_string(scope, arguments.get(0));
    let snapshot = record(scope, arguments.this());
    let Some(snapshot) = snapshot else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let length = text_len(&snapshot.value);
    let (start, end) = if arguments.length() >= 3 {
        (
            arguments
                .get(1)
                .uint32_value(scope)
                .unwrap_or(0)
                .min(length),
            arguments
                .get(2)
                .uint32_value(scope)
                .unwrap_or(0)
                .min(length),
        )
    } else {
        (snapshot.selection_start, snapshot.selection_end)
    };
    if start > end {
        crate::webidl::throw_type_error(scope, "The start index exceeds the end index");
        return;
    }
    let mode = if arguments.length() >= 4 {
        crate::webidl::value_to_string(scope, arguments.get(3))
    } else {
        "preserve".to_owned()
    };
    let chars = snapshot.value.chars().collect::<Vec<_>>();
    let safe_start = (start as usize).min(chars.len());
    let safe_end = (end as usize).min(chars.len());
    let before = chars[..safe_start].iter().collect::<String>();
    let after = chars[safe_end..].iter().collect::<String>();
    let value = format!("{before}{replacement}{after}");
    let inserted_end = start.saturating_add(text_len(&replacement));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.value = value;
        match mode.as_str() {
            "select" => {
                record.selection_start = start;
                record.selection_end = inserted_end;
            }
            "start" => {
                record.selection_start = start;
                record.selection_end = start;
            }
            "end" => {
                record.selection_start = inserted_end;
                record.selection_end = inserted_end;
            }
            _ => {
                let removed = end - start;
                let inserted = text_len(&replacement);
                record.selection_start =
                    adjust_position(snapshot.selection_start, start, end, inserted, removed);
                record.selection_end =
                    adjust_position(snapshot.selection_end, start, end, inserted, removed);
            }
        }
    }
}
