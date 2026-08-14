pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "selectNode", 1, select_node)
}
fn select_node(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::abstract_range::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    let Some(parent) = super::node::parent(scope, node) else {
        crate::webidl::throw_type_error(scope, "The node has no parent");
        return;
    };
    let children = super::node::children(scope, parent);
    let Some(index) = children
        .iter()
        .position(|child| child.strict_equals(node.into()))
    else {
        return;
    };
    let parent = v8::Global::new(scope, parent);
    super::abstract_range::update(scope, arguments.this(), |range| {
        range.start_container = parent.clone();
        range.start_offset = index as u32;
        range.end_container = parent;
        range.end_offset = index as u32 + 1;
    });
}
