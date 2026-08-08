use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getCharNumAtPosition",
        0,
        get_char_num_at_position,
    )
}

fn get_char_num_at_position(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let x = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|point| {
            let key = v8::String::new(scope, "x")?;
            point.get(scope, key.into())?.number_value(scope)
        })
        .unwrap_or(-1.0);
    let index = (x / 10.0).floor() as i32;
    let output = if index >= 0 && index < glyph_count(&record.text) as i32 {
        index
    } else {
        -1
    };
    result.set(v8::Integer::new(scope, output).into());
}
