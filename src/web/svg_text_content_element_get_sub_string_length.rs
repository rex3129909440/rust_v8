use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getSubStringLength",
        2,
        get_sub_string_length,
    )
}

fn get_sub_string_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let start = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let count = arguments.get(1).uint32_value(scope).unwrap_or(0) as usize;
    let total = glyph_count(&record.text);
    if start >= total && count != 0 {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    }
    result.set(v8::Number::new(scope, count.min(total.saturating_sub(start)) as f64 * 10.0).into());
}
