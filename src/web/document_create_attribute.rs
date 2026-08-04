pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createAttribute", 1, create_attribute)
}

fn create_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::document::valid_xml_name(&input) {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "The attribute name is not a valid XML name",
        );
        return;
    }
    let name = if super::document::content_type(scope, arguments.this()) == Some("text/html") {
        input.to_ascii_lowercase()
    } else {
        input
    };
    match super::attr::create(scope, name, String::new(), None, None) {
        Ok(attribute) => {
            super::node::set_owner_document(scope, attribute, arguments.this());
            result.set(attribute.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
