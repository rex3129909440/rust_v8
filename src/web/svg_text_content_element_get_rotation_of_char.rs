use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getRotationOfChar",
        1,
        get_rotation_of_char,
    )
}

fn get_rotation_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if character_index(scope, &arguments).is_some() {
        result.set(v8::Number::new(scope, 0.0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
    }
}
