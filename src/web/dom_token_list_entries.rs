pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)
}
fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    crate::webidl::return_array_like_iterator(
        scope,
        arguments.this(),
        crate::webidl::ArrayLikeIteratorKind::Entries,
        result,
    )
}
