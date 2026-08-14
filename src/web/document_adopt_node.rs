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
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'adoptNode' on 'Document': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'adoptNode' on 'Document': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    let Some(record) = super::node::record(scope, node) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'adoptNode' on 'Document': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    let old_document = record
        .owner_document
        .as_ref()
        .map(|document| v8::Local::new(scope, document));
    if record.node_type == super::node::DOCUMENT_NODE {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Failed to execute 'adoptNode' on 'Document': The node provided is of type '#document', which may not be adopted.",
        );
        return;
    }
    if super::shadow_root::host(scope, node).is_some() {
        super::node::throw_dom_exception(
            scope,
            "HierarchyRequestError",
            "Failed to execute 'adoptNode' on 'Document': The node provided is a shadow root, which may not be adopted.",
        );
        return;
    }
    if record.node_type == super::node::ATTRIBUTE_NODE {
        if let Some(attribute) = super::attr::record(scope, node)
            && let Some(owner) = attribute.owner_element
        {
            let owner = v8::Local::new(scope, &owner);
            super::element::remove_attribute_full(
                scope,
                owner,
                attribute.namespace_uri.as_deref(),
                &attribute.local_name,
            );
            super::attr::set_owner(scope, node, None);
        }
    } else {
        super::node::detach(scope, node);
    }
    super::node::set_owner_document_recursive(scope, node, arguments.this());
    if let Some(old_document) = old_document {
        super::custom_element_registry::notify_adopted_tree(
            scope,
            node,
            old_document,
            arguments.this(),
        );
    }
    result.set(node.into());
}
