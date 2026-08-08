pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "empty", 0, empty)
}
fn empty(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::selection::update(scope, a.this(), |v| {
        v.anchor = None;
        v.focus = None;
        v.ranges.clear();
        v.direction = "none".to_owned();
    })
}
