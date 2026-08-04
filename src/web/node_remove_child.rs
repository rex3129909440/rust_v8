pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "removeChild", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(child) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "removeChild requires a Node");
        return;
    };
    if super::node::parent(scope, child)
        .is_some_and(|parent| parent.strict_equals(arguments.this().into()))
        && super::node::detach(scope, child)
    {
        result.set(child.into());
    } else {
        super::node::throw_dom_exception(scope, "NotFoundError", "The node is not a child");
    }
}
