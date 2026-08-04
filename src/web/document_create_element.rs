pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createElement", 1, create_element)
}

fn create_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::document::valid_xml_name(&name) {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "The tag name provided is not a valid name",
        );
        return;
    }
    let normalized = name.to_ascii_lowercase();
    match super::document::create_html_element_by_name(scope, &normalized) {
        Ok(element) => {
            super::node::set_owner_document(scope, element, arguments.this());
            result.set(element.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
