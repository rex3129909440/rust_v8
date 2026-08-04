use super::html_canvas_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toBlob", 1, to_blob)
}

fn to_blob(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The callback must be a function");
        return;
    };
    if snapshot.width == 0 || snapshot.height == 0 {
        let _ = callback.call(
            scope,
            v8::undefined(scope).into(),
            &[v8::null(scope).into()],
        );
        return;
    }
    let Some(bytes) = fingerprinted_png_bytes(
        scope,
        snapshot.width,
        snapshot.height,
        canvas_pixels(scope, &snapshot).as_deref(),
    ) else {
        let _ = callback.call(
            scope,
            v8::undefined(scope).into(),
            &[v8::null(scope).into()],
        );
        return;
    };
    match super::blob::create(scope, bytes, "image/png") {
        Ok(blob) => {
            let _ = callback.call(scope, v8::undefined(scope).into(), &[blob.into()]);
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
