pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "modify", 0, modify)
}
fn modify(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(focus) = current.focus else {
        return;
    };
    let alter = crate::webidl::value_to_string(scope, a.get(0)).to_ascii_lowercase();
    let direction = crate::webidl::value_to_string(scope, a.get(1)).to_ascii_lowercase();
    let granularity = crate::webidl::value_to_string(scope, a.get(2)).to_ascii_lowercase();
    if !matches!(alter.as_str(), "move" | "extend") {
        return;
    }
    let backward = matches!(direction.as_str(), "backward" | "left");
    if !backward && !matches!(direction.as_str(), "forward" | "right") {
        return;
    }
    let focus_local = v8::Local::new(scope, &focus);
    let maximum = super::range::boundary_length(scope, focus_local).unwrap_or(0);
    let mut offset = current.focus_offset.min(maximum);
    match granularity.as_str() {
        "character" => {
            offset = if backward {
                offset.saturating_sub(1)
            } else {
                offset.saturating_add(1).min(maximum)
            };
        }
        "word" => {
            let units = super::character_data::data_if_character(scope, focus_local)
                .map(|value| value.encode_utf16().collect::<Vec<_>>())
                .unwrap_or_default();
            if units.is_empty() {
                offset = if backward { 0 } else { maximum };
            } else if backward {
                let mut index = offset as usize;
                while index > 0
                    && char::from_u32(units[index - 1] as u32).is_some_and(char::is_whitespace)
                {
                    index -= 1;
                }
                while index > 0
                    && !char::from_u32(units[index - 1] as u32).is_some_and(char::is_whitespace)
                {
                    index -= 1;
                }
                offset = index as u32;
            } else {
                let mut index = offset as usize;
                while index < units.len()
                    && !char::from_u32(units[index] as u32).is_some_and(char::is_whitespace)
                {
                    index += 1;
                }
                while index < units.len()
                    && char::from_u32(units[index] as u32).is_some_and(char::is_whitespace)
                {
                    index += 1;
                }
                offset = index as u32;
            }
        }
        "lineboundary" | "sentenceboundary" | "documentboundary" => {
            offset = if backward { 0 } else { maximum };
        }
        "line" | "sentence" | "paragraph" | "paragraphboundary" => {
            offset = if backward { 0 } else { maximum };
        }
        _ => return,
    }
    if alter == "move" {
        let range =
            super::selection::selection_range(scope, focus_local, offset, focus_local, offset);
        let anchor = v8::Global::new(scope, focus_local);
        let focus = v8::Global::new(scope, focus_local);
        super::selection::update(scope, a.this(), |selection| {
            selection.anchor = Some(anchor);
            selection.focus = Some(focus);
            selection.anchor_offset = offset;
            selection.focus_offset = offset;
            selection.ranges = range.into_iter().collect();
            selection.direction = "none".to_owned();
        });
        return;
    }
    let Some(anchor) = current.anchor else {
        return;
    };
    let anchor_local = v8::Local::new(scope, &anchor);
    let range = super::selection::selection_range(
        scope,
        anchor_local,
        current.anchor_offset,
        focus_local,
        offset,
    );
    let selection_direction = super::selection::direction_between(
        scope,
        anchor_local,
        current.anchor_offset,
        focus_local,
        offset,
    );
    let focus = v8::Global::new(scope, focus_local);
    super::selection::update(scope, a.this(), |selection| {
        selection.focus = Some(focus);
        selection.focus_offset = offset;
        selection.ranges = range.into_iter().collect();
        selection.direction = selection_direction;
    });
}
