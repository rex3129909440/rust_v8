use super::svg_text_content_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getStartPositionOfChar",
        1,
        get_start_position_of_char,
    )
}

fn get_start_position_of_char(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some((_, index)) = character_index(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Character index is out of bounds");
        return;
    };
    return_point(scope, index as f64 * 10.0, 0.0, result);
}
