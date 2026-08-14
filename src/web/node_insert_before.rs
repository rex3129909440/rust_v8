pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "insertBefore", 2, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(child) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "insertBefore requires a Node");
        return;
    };
    let children = super::node::children(scope, arguments.this());
    let index = if arguments.get(1).is_null() {
        children.len()
    } else {
        let Ok(reference) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
            crate::webidl::throw_type_error(scope, "The reference must be a Node or null");
            return;
        };
        let Some(index) = children
            .iter()
            .position(|node| node.strict_equals(reference.into()))
        else {
            super::node::throw_dom_exception(
                scope,
                "NotFoundError",
                "The reference node is not a child",
            );
            return;
        };
        index
    };
    match super::node::insert_node(scope, arguments.this(), child, index) {
        Ok(()) => result.set(child.into()),
        Err((name, message)) => super::node::throw_dom_exception(scope, name, message),
    }
}
