pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "insertAdjacentHTML",
        2,
        insert_adjacent_html,
    )
}

fn insert_adjacent_html(
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
    let html = crate::webidl::value_to_string(scope, arguments.get(1));
    let parsed = match super::dom_html::parse_fragment(scope, parent, &html) {
        Ok(parsed) => parsed,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Err(error) = super::dom_nodes::insert(scope, parent, index, &parsed) {
        super::dom_nodes::insert_error(scope, error);
    }
}
