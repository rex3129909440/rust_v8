use super::svg_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "crossOrigin",
        get_cross_origin,
        set_cross_origin,
    )
}

fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        if let Some(value) = value.cross_origin {
            return_string(scope, &value, result);
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let normalized = if value.is_null() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, value);
        Some(if value == "use-credentials" {
            value
        } else {
            "anonymous".to_owned()
        })
    };
    update(scope, arguments.this(), |record| {
        record.cross_origin = normalized
    });
}
