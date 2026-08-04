pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "replaceWith", 0, call)
}
fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::document_type::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(parent) = super::node::parent(scope, arguments.this()) else {
        return;
    };
    let Some(index) = super::dom_nodes::child_index(scope, parent, arguments.this()) else {
        return;
    };
    let nodes = match super::dom_nodes::arguments(scope, &arguments) {
        Ok(nodes) => nodes,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Err(error) = super::dom_nodes::insert(scope, parent, index, &nodes) {
        super::dom_nodes::insert_error(scope, error);
        return;
    }
    if super::node::parent(scope, arguments.this())
        .is_some_and(|current| current.strict_equals(parent.into()))
    {
        super::node::detach(scope, arguments.this());
    }
}
