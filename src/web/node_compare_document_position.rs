pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "compareDocumentPosition", 1, call)
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
    let Ok(other) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Argument is not a Node");
        return;
    };
    if super::node::record(scope, other).is_none() {
        crate::webidl::throw_type_error(scope, "Argument is not a Node");
        return;
    }
    let this = arguments.this();
    let code = if this.strict_equals(other.into()) {
        0
    } else if super::node::is_descendant(scope, this, other) {
        super::node::DOCUMENT_POSITION_CONTAINED_BY | super::node::DOCUMENT_POSITION_FOLLOWING
    } else if super::node::is_descendant(scope, other, this) {
        super::node::DOCUMENT_POSITION_CONTAINS | super::node::DOCUMENT_POSITION_PRECEDING
    } else {
        let this_root = super::node::root_node(scope, this);
        let other_root = super::node::root_node(scope, other);
        if this_root.get_identity_hash().get() != other_root.get_identity_hash().get() {
            let direction = if this.get_identity_hash().get() < other.get_identity_hash().get() {
                super::node::DOCUMENT_POSITION_FOLLOWING
            } else {
                super::node::DOCUMENT_POSITION_PRECEDING
            };
            super::node::DOCUMENT_POSITION_DISCONNECTED
                | super::node::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
                | direction
        } else {
            let order = super::node::tree_order(scope, this_root);
            let this_id = this.get_identity_hash().get();
            let other_id = other.get_identity_hash().get();
            let this_index = order
                .iter()
                .position(|node| node.get_identity_hash().get() == this_id)
                .unwrap_or(0);
            let other_index = order
                .iter()
                .position(|node| node.get_identity_hash().get() == other_id)
                .unwrap_or(0);
            if this_index < other_index {
                super::node::DOCUMENT_POSITION_FOLLOWING
            } else {
                super::node::DOCUMENT_POSITION_PRECEDING
            }
        }
    };
    result.set(v8::Integer::new(scope, code).into());
}
