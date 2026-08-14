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
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createAttribute' on 'Document': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(input) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'createAttribute' on 'Document'",
    ) else {
        return;
    };
    if !super::document::valid_xml_name(&input) {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            &format!(
                "Failed to execute 'createAttribute' on 'Document': The localName provided ('{input}') contains an invalid character."
            ),
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
