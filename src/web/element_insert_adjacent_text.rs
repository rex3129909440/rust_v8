pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "insertAdjacentText",
        2,
        insert_adjacent_text,
    )
}

fn insert_adjacent_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let position = crate::webidl::value_to_string(scope, arguments.get(0));
    let location = match super::dom_nodes::adjacent_location(scope, arguments.this(), &position) {
        Ok(location) => location,
        Err(()) => {
            super::dom_nodes::throw_invalid_adjacent_position(scope);
            return;
        }
    };
    let Some((parent, index)) = location else {
        return;
    };
    let text = crate::webidl::value_to_string(scope, arguments.get(1));
    let Ok(text) = super::text::create(scope, text) else {
        return;
    };
    if let Err(error) = super::node::insert_node(scope, parent, text, index) {
        super::dom_nodes::insert_error(scope, error);
    }
}
