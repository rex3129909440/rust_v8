use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getExtentOfChar", 1, get_extent_of_char)
}

fn get_extent_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some((_, index)) = character_index(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    };
    match super::svg_rect::create_pair(
        scope,
        super::svg_rect::RectValue {
            x: index as f64 * 10.0,
            y: -10.0,
            width: 10.0,
            height: 12.0,
        },
    ) {
        Ok((rect, _)) => result.set(rect.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
