pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "adoptNode", 1, adopt_node)
}

fn adopt_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The provided value is not a Node");
        return;
    };
    let Some(record) = super::node::record(scope, node) else {
        crate::webidl::throw_type_error(scope, "The provided value is not a Node");
        return;
    };
    if record.node_type == super::node::DOCUMENT_NODE {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Document nodes cannot be adopted",
        );
        return;
    }
    if super::shadow_root::host(scope, node).is_some() {
        super::node::throw_dom_exception(
            scope,
            "HierarchyRequestError",
            "ShadowRoot nodes cannot be adopted",
        );
        return;
    }
    super::node::detach(scope, node);
    super::node::set_owner_document_recursive(scope, node, arguments.this());
    result.set(node.into());
}
