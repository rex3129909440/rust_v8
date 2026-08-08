pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)
}
fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = if v.ranges.is_empty() {
        "None"
    } else if v.anchor_offset == v.focus_offset {
        "Caret"
    } else {
        "Range"
    };
    if let Some(s) = v8::String::new(scope, value) {
        r.set(s.into())
    }
}
