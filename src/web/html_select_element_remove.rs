use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "remove", 0, remove)
}

fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if a.get(0).is_undefined() {
        let _ = super::node::detach(scope, a.this());
    } else {
        let i = a.get(0).int32_value(scope).unwrap_or(-1);
        remove_option_index(scope, a.this(), i)
    }
}
