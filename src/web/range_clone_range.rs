pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "cloneRange", 0, clone_range)
}
fn clone_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    if let Ok(range) = super::range::create_from_record(scope, record) {
        result.set(range.into());
    }
}
