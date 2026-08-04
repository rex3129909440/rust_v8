use super::html_option_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "form", get_form)
}

fn get_form(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(form) =
        super::html_select_element::containing_select(scope, arguments.this())
            .and_then(|select| super::html_form_element::ancestor_form(scope, select))
    {
        result.set(form.into());
    } else {
        result.set(v8::null(scope).into());
    }
}
