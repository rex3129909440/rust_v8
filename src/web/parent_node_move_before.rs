pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "moveBefore", 2, move_before)
}

fn move_before(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::dom_nodes::ensure_parent_node(scope, arguments.this()) {
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The first argument must be a Node");
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(scope, "The first argument must be a Node");
        return;
    }
    let reference = if arguments.get(1).is_null() {
        None
    } else {
        let Ok(reference) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
            crate::webidl::throw_type_error(scope, "The second argument must be a Node or null");
            return;
        };
        Some(reference)
    };
    if reference.is_some_and(|reference| reference.strict_equals(node.into())) {
        return;
    }
    let children = super::node::children(scope, arguments.this());
    let index = match reference {
        Some(reference) => {
            let Some(index) = children
                .iter()
                .position(|child| child.strict_equals(reference.into()))
            else {
                super::node::throw_dom_exception(
                    scope,
                    "NotFoundError",
                    "The reference node is not a child of this node",
                );
                return;
            };
            index
        }
        None => children.len(),
    };
    if let Err((name, message)) = super::node::insert_node(scope, arguments.this(), node, index) {
        super::node::throw_dom_exception(scope, name, message);
    }
}
