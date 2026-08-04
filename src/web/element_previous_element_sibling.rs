pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "previousElementSibling",
        get_previous_element_sibling,
    )
}

fn get_previous_element_sibling(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::node::parent(scope, arguments.this()).and_then(|parent| {
        let elements = super::node::children(scope, parent)
            .into_iter()
            .filter(|child| super::element::record(scope, *child).is_some())
            .collect::<Vec<_>>();
        let index = elements
            .iter()
            .position(|child| child.strict_equals(arguments.this().into()))?;
        index
            .checked_sub(1)
            .and_then(|position| elements.get(position).copied())
    });
    match value {
        Some(sibling) => result.set(sibling.into()),
        None => result.set(v8::null(scope).into()),
    }
}
