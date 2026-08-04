use super::html_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "lowsrc", get_low_src, set_low_src)
}

fn get_low_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else {
        let value = super::element::resolved_url_attribute(scope, arguments.this(), "lowsrc")
            .unwrap_or_default();
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    }
}

fn set_low_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(scope, arguments, "lowsrc");
}
