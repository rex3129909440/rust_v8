pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "currentNode",
        get_current_node,
        set_current_node,
    )
}
fn get_current_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = super::tree_walker::record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.current).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_current_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::tree_walker::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "currentNode must be a Node");
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(scope, "currentNode must be a Node");
        return;
    }
    super::tree_walker::set_current(scope, arguments.this(), node);
}
