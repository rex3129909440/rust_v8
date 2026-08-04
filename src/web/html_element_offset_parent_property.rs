use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "offsetParent", get_offset_parent)
}

pub(crate) fn get_offset_parent(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        match super::element_layout::offset_parent(scope, arguments.this()) {
            Some(parent) => result.set(parent.into()),
            None => result.set(v8::null(scope).into()),
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
