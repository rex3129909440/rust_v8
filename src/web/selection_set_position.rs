pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setPosition", 1, set_position)
}

fn set_position(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.get(0).is_null() {
        super::selection::update(scope, arguments.this(), |selection| {
            selection.anchor = None;
            selection.focus = None;
            selection.ranges.clear();
            selection.direction = "none".to_owned();
        });
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "setPosition requires a Node");
        return;
    };
    let offset = arguments.get(1).uint32_value(scope).unwrap_or(0);
    if !super::selection::valid_offset(scope, node, offset) {
        super::node::throw_dom_exception(scope, "IndexSizeError", "The offset is out of bounds");
        return;
    }
    let range = super::selection::selection_range(scope, node, offset, node, offset);
    let anchor = v8::Global::new(scope, node);
    let focus = v8::Global::new(scope, node);
    super::selection::update(scope, arguments.this(), |selection| {
        selection.anchor = Some(anchor);
        selection.focus = Some(focus);
        selection.anchor_offset = offset;
        selection.focus_offset = offset;
        selection.ranges = range.into_iter().collect();
        selection.direction = "none".to_owned();
    });
}
