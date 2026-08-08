use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createSVGLength", 0, create_svg_length)
}

fn create_svg_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_length::create_single(
        scope,
        super::svg_length::LengthSnapshot {
            unit: 1,
            specified: 0.0,
        },
    );
    return_created(scope, value, result);
}
