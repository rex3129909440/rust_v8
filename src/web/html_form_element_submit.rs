use super::html_form_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "submit", 0, submit)
}

fn submit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        update(scope, arguments.this(), |record| record.submit_count += 1);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
