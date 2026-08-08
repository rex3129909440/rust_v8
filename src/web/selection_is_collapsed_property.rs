pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "isCollapsed", get_is_collapsed)
}
fn get_is_collapsed(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let same = match (v.anchor, v.focus) {
        (Some(x), Some(y)) => {
            v8::Local::new(scope, &x).strict_equals(v8::Local::new(scope, &y).into())
                && v.anchor_offset == v.focus_offset
        }
        _ => true,
    };
    r.set(v8::Boolean::new(scope, same).into())
}
