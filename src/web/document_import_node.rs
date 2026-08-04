pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "importNode", 1, import_node)
}

fn import_node(
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
    if record.node_type == super::node::DOCUMENT_NODE
        || super::shadow_root::host(scope, node).is_some()
    {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Document and ShadowRoot nodes cannot be imported",
        );
        return;
    }
    let deep = arguments.get(1).boolean_value(scope);
    match super::node::clone_object(scope, node, deep) {
        Ok(clone) => {
            super::node::set_owner_document_recursive(scope, clone, arguments.this());
            result.set(clone.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
