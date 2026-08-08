use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createSVGTransformFromMatrix",
        1,
        create_svg_transform_from_matrix,
    )
}

fn create_svg_transform_from_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let matrix = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|matrix| super::svg_matrix::value(scope, matrix));
    let Some(matrix) = matrix else {
        crate::webidl::throw_type_error(scope, "An SVGMatrix is required");
        return;
    };
    let value = super::svg_transform::create(
        scope,
        super::svg_transform::TransformValue {
            kind: 1,
            matrix,
            angle: 0.0,
        },
    );
    return_created(scope, value, result);
}
