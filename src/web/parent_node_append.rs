pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "append", 0, append)
}

fn append(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::dom_nodes::ensure_parent_node(scope, arguments.this()) {
        return;
    }
    let nodes = match super::dom_nodes::arguments(scope, &arguments) {
        Ok(nodes) => nodes,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let index = super::node::children(scope, arguments.this()).len();
    if let Err(error) = super::dom_nodes::insert(scope, arguments.this(), index, &nodes) {
        super::dom_nodes::insert_error(scope, error);
    }
}
