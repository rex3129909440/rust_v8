pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "prepend", 0, prepend)
}

fn prepend(
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
    if let Err(error) = super::dom_nodes::insert(scope, arguments.this(), 0, &nodes) {
        super::dom_nodes::insert_error(scope, error);
    }
}
