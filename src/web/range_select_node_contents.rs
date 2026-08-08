pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "selectNodeContents",
        1,
        select_node_contents,
    )
}
fn select_node_contents(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    let Some(length) = super::range::boundary_length(scope, node) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    if super::node::record(scope, node).is_some_and(|record| record.node_type == 10) {
        super::node::throw_dom_exception(
            scope,
            "InvalidNodeTypeError",
            "DocumentType nodes cannot be range boundary containers",
        );
        return;
    }
    let node = v8::Global::new(scope, node);
    super::abstract_range::update(scope, arguments.this(), |range| {
        range.start_container = node.clone();
        range.start_offset = 0;
        range.end_container = node;
        range.end_offset = length;
    });
}
