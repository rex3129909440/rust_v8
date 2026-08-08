pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setStart", 2, set_start)
}
fn set_start(
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
    let end = v8::Local::new(scope, &current.end_container);
    let collapse =
        super::range::compare_boundaries(scope, node_local, offset, end, current.end_offset)
            .is_none_or(|ordering| ordering > 0);
    super::abstract_range::update(scope, arguments.this(), |range| {
        range.start_container = node.clone();
        range.start_offset = offset;
        if collapse {
            range.end_container = node;
            range.end_offset = offset;
        }
    });
}
