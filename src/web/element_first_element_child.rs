pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "firstElementChild",
        get_first_element_child,
    )
}

fn get_first_element_child(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::node::children(scope, arguments.this())
        .into_iter()
        .find(|child| super::element::record(scope, *child).is_some())
    {
        Some(child) => result.set(child.into()),
        None => result.set(v8::null(scope).into()),
    }
}
