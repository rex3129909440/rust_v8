pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "parentNode", 0, parent_node)
}
fn parent_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::tree_walker::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let current = v8::Local::new(scope, &record.current);
    let root = v8::Local::new(scope, &record.root);
    let candidate = if current.strict_equals(root.into()) {
        None
    } else {
        match super::tree_walker::visible_parent(scope, &record, current) {
            Ok(value) => value.map(|value| v8::Local::new(scope, &value)),
            Err(()) => return,
        }
    };
    super::tree_walker::return_candidate(scope, arguments.this(), candidate, result);
}
