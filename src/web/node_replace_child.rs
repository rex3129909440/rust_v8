pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "replaceChild", 2, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let (Ok(new_child), Ok(old_child)) = (
        v8::Local::<v8::Object>::try_from(arguments.get(0)),
        v8::Local::<v8::Object>::try_from(arguments.get(1)),
    ) else {
        crate::webidl::throw_type_error(scope, "replaceChild requires Nodes");
        return;
    };
    let children = super::node::children(scope, arguments.this());
    let Some(index) = children
        .iter()
        .position(|node| node.strict_equals(old_child.into()))
    else {
        super::node::throw_dom_exception(scope, "NotFoundError", "The node is not a child");
        return;
    };
    if new_child.strict_equals(old_child.into()) {
        result.set(old_child.into());
        return;
    }
    match super::node::insert_node(scope, arguments.this(), new_child, index) {
        Ok(()) => {
            super::node::detach(scope, old_child);
            result.set(old_child.into());
        }
        Err((name, message)) => super::node::throw_dom_exception(scope, name, message),
    }
}
