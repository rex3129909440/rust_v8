pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "outerHTML", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let html = super::dom_html::serialize_node(scope, arguments.this());
    if let Some(html) = v8::String::new(scope, &html) {
        result.set(html.into());
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(parent) = super::node::parent(scope, arguments.this()) else {
        return;
    };
    if super::node::record(scope, parent).is_some_and(|record| record.node_type == 9) {
        super::node::throw_dom_exception(
            scope,
            "NoModificationAllowedError",
            "The parent node is a Document",
        );
        return;
    }
    let siblings = super::node::children(scope, parent);
    let Some(index) = siblings.iter().position(|node| {
        node.get_identity_hash().get() == arguments.this().get_identity_hash().get()
    }) else {
        return;
    };
    let html = crate::webidl::value_to_string(scope, arguments.get(0));
    let parsed = match super::dom_html::parse_fragment(scope, parent, &html) {
        Ok(parsed) => parsed,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    for (offset, node) in parsed.iter().enumerate() {
        if let Err((name, message)) =
            super::node::insert_node(scope, parent, v8::Local::new(scope, node), index + offset)
        {
            super::node::throw_dom_exception(scope, name, message);
            return;
        }
    }
    super::node::detach(scope, arguments.this());
}
