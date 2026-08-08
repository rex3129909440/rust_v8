use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getComputedTextLength",
        0,
        get_computed_text_length,
    )
}

fn get_computed_text_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, glyph_count(&record.text) as f64 * 10.0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
