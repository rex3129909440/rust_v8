use super::svg_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "decode", 0, decode)
}

fn decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "SVGImageElement",
            "decode",
            result,
        );
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        crate::webidl::throw_type_error(scope, "Cannot create decode promise");
        return;
    };
    let promise = resolver.get_promise(scope);
    let _ = resolver.resolve(scope, v8::undefined(scope).into());
    result.set(promise.into());
}
