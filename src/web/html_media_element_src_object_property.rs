use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "srcObject",
        get_src_object,
        set_src_object,
    )
}

fn get_src_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(object) = record.src_object {
            result.set(v8::Local::new(scope, &object).into());
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_src_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let object = if value.is_null() {
        None
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(scope, "srcObject must be a MediaStream or null");
            return;
        };
        if !super::media_stream::is_stream(scope, object) {
            crate::webidl::throw_type_error(scope, "srcObject must be a MediaStream or null");
            return;
        }
        Some(v8::Global::new(scope, object))
    };
    update(scope, arguments.this(), |record| record.src_object = object);
}
