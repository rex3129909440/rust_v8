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
    let Some(record) = super::tree_walker::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let root = v8::Local::new(scope, &record.root);
    let current = v8::Local::new(scope, &record.current);
    if current.strict_equals(root.into()) {
        result.set(v8::null(scope).into());
        return;
    }
    let mut traversal = vec![(v8::Global::new(scope, root), true)];
    if super::tree_walker::traversable_preorder(scope, &record, root, &mut traversal).is_err() {
        return;
    }
    let position = traversal
        .iter()
        .position(|(node, _)| v8::Local::new(scope, node).strict_equals(current.into()))
        .unwrap_or(traversal.len());
    let candidate = traversal[..position]
        .iter()
        .rev()
        .find_map(|(node, accepted)| accepted.then(|| v8::Local::new(scope, node)));
    super::tree_walker::return_candidate(scope, arguments.this(), candidate, result);
}
