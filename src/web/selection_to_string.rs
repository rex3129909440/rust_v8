pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)
}
fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut text = String::new();
    for range in v.ranges {
        let range = v8::Local::new(scope, &range);
        if let Some(selected) = super::range::selected_text(scope, range) {
            text.push_str(&selected);
        }
    }
    if let Some(s) = v8::String::new(scope, &text) {
        r.set(s.into())
    }
}
