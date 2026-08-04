pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setEnd", 2, set_end)
}
fn set_end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = super::range::record_or_throw(scope, arguments.this()) else {
        return;
    };
    let Some((node, offset)) = super::range::boundary_arguments(scope, &arguments) else {
        return;
    };
    let node_local = v8::Local::new(scope, &node);
    let start = v8::Local::new(scope, &current.start_container);
    let collapse =
        super::range::compare_boundaries(scope, node_local, offset, start, current.start_offset)
            .is_none_or(|ordering| ordering < 0);
    super::abstract_range::update(scope, arguments.this(), |range| {
        range.end_container = node.clone();
        range.end_offset = offset;
        if collapse {
            range.start_container = node;
            range.start_offset = offset;
        }
    });
}
