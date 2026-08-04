pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "previousNode", 0, previous_node)
}
fn previous_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = super::node_iterator::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let root = v8::Local::new(scope, &snapshot.root);
    let reference = v8::Local::new(scope, &snapshot.reference);
    let mut nodes = Vec::new();
    super::node_iterator::collect_nodes(scope, root, &mut nodes);
    let mut index = nodes
        .iter()
        .position(|node| node.strict_equals(reference.into()))
        .unwrap_or(0) as isize;
    if snapshot.pointer_before_reference_node {
        index -= 1;
    }
    while index >= 0 {
        let position = index as usize;
        let node = nodes[position];
        super::node_iterator::update_position(scope, arguments.this(), node, true);
        match super::node_iterator::accepts(scope, &snapshot, node) {
            Ok(true) => {
                result.set(node.into());
                return;
            }
            Ok(false) => {}
            Err(()) => return,
        }
        index -= 1;
    }
    result.set(v8::null(scope).into());
}
