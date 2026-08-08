pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "insertAdjacentElement",
        2,
        insert_adjacent_element,
    )
}

fn insert_adjacent_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
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
    let Ok(element) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The second argument must be an Element");
        return;
    };
    if super::element::record(scope, element).is_none() {
        crate::webidl::throw_type_error(scope, "The second argument must be an Element");
        return;
    }
    let Some((parent, index)) = location else {
        result.set(v8::null(scope).into());
        return;
    };
    match super::node::insert_node(scope, parent, element, index) {
        Ok(()) => result.set(element.into()),
        Err(error) => super::dom_nodes::insert_error(scope, error),
    }
}
