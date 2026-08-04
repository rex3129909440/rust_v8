pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "currentCSSZoom",
        get_current_css_zoom,
    )
}

fn get_current_css_zoom(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_some() {
        result.set(v8::Number::new(scope, 1.0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
