pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "nextNode", 0, next_node)
}
fn next_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = super::node_iterator::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let original_reference = snapshot.reference.clone();
    let original_before = snapshot.pointer_before_reference_node;
    loop {
        let Some(current) = super::node_iterator::record(scope, arguments.this()) else {
            return;
        };
        let root = v8::Local::new(scope, &current.root);
        let reference = v8::Local::new(scope, &current.reference);
        let candidate = if current.pointer_before_reference_node {
            Some(reference)
        } else {
            super::node_iterator::following_node(scope, reference, root)
        };
        let Some(node) = candidate else {
            let original = v8::Local::new(scope, &original_reference);
            super::node_iterator::update_position(
                scope,
                arguments.this(),
                original,
                original_before,
            );
            result.set(v8::null(scope).into());
            return;
        };
        super::node_iterator::update_position(scope, arguments.this(), node, false);
        match super::node_iterator::accepts(scope, &snapshot, node, "nextNode") {
            Ok(true) => {
                result.set(node.into());
                return;
            }
            Ok(false) => {}
            Err(()) => {
                let original = v8::Local::new(scope, &original_reference);
                super::node_iterator::update_position(
                    scope,
                    arguments.this(),
                    original,
                    original_before,
                );
                return;
            }
        }
    }
}
