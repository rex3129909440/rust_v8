pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getElementById", 1, get_element_by_id)
}

fn get_element_by_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let wanted = crate::webidl::value_to_string(scope, arguments.get(0));
    let found = super::dom_selector::descendants(scope, arguments.this())
        .into_iter()
        .find(|element| {
            super::element::attributes_snapshot(scope, *element).is_some_and(|attributes| {
                attributes
                    .into_iter()
                    .any(|attribute| attribute.name == "id" && attribute.value == wanted)
            })
        });
    match found {
        Some(element) => result.set(element.into()),
        None => result.set(v8::null(scope).into()),
    }
}
