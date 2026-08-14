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
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
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
    if super::node::parent(scope, new_child)
        .is_some_and(|parent| parent.strict_equals(arguments.this().into()))
    {
        // Replacing with an existing child first removes that child as a
        // separate tree mutation, then performs the replacement mutation.
        super::node::detach(scope, new_child);
    }
    let children = super::node::children(scope, arguments.this());
    let Some(index) = children
        .iter()
        .position(|node| node.strict_equals(old_child.into()))
    else {
        super::node::throw_dom_exception(scope, "NotFoundError", "The node is not a child");
        return;
    };
    let previous_sibling = index
        .checked_sub(1)
        .and_then(|index| children.get(index).copied());
    let next_sibling = children.get(index + 1).copied();
    let added_nodes =
        if super::node::record(scope, new_child).is_some_and(|record| record.node_type == 11) {
            super::node::children(scope, new_child)
        } else {
            vec![new_child]
        };
    super::mutation_observer::suppress_child_list_for(scope, arguments.this());
    let inserted = super::node::insert_node(scope, arguments.this(), new_child, index);
    match inserted {
        Ok(()) => {
            super::node::detach(scope, old_child);
            super::mutation_observer::unsuppress_child_list_for(scope, arguments.this());
            super::mutation_observer::enqueue_child_list(
                scope,
                arguments.this(),
                added_nodes,
                vec![old_child],
                previous_sibling,
                next_sibling,
            );
            result.set(old_child.into());
        }
        Err((name, message)) => {
            super::mutation_observer::unsuppress_child_list_for(scope, arguments.this());
            super::node::throw_dom_exception(scope, name, message);
        }
    }
}
