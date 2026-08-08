use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getElementById", 1, get_element_by_id)
}

fn get_element_by_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let expected = crate::webidl::value_to_string(scope, arguments.get(0));
    let mut pending = super::node::children(scope, arguments.this());
    while let Some(candidate) = pending.pop() {
        if element_has_id(scope, candidate, &expected) {
            result.set(candidate.into());
            return;
        }
        pending.extend(super::node::children(scope, candidate));
    }
    result.set(v8::null(scope).into());
}
