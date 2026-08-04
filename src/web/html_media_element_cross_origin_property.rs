use super::html_media_element::*;

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
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(cross_origin) = record.cross_origin {
            return_string(scope, &mut result, &cross_origin);
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
    let value = if arguments.get(0).is_null() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        Some(if value == "use-credentials" {
            value
        } else {
            "anonymous".to_owned()
        })
    };
    update(scope, arguments.this(), |record| {
        record.cross_origin = value
    });
}
