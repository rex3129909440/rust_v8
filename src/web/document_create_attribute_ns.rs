pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createAttributeNS",
        2,
        create_attribute_ns,
    )
}

fn create_attribute_ns(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'createAttributeNS' on 'Document': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let namespace = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        let Some(namespace) = crate::webidl::dom_string_with_context(
            scope,
            arguments.get(0),
            "Failed to execute 'createAttributeNS' on 'Document'",
        ) else {
            return;
        };
        (!namespace.is_empty()).then_some(namespace)
    };
    let Some(qualified_name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(1),
        "Failed to execute 'createAttributeNS' on 'Document'",
    ) else {
        return;
    };
    if let Err((name, _)) =
        super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, true)
    {
        let message = super::document::qualified_name_error_message(
            "createAttributeNS",
            "Document",
            name,
            namespace.as_deref(),
            &qualified_name,
        );
        super::node::throw_dom_exception(scope, name, &message);
        return;
    }
    let qualified_name = super::document::canonical_qualified_name(&qualified_name);
    match super::attr::create(scope, qualified_name, String::new(), namespace, None) {
        Ok(attribute) => {
            super::node::set_owner_document(scope, attribute, arguments.this());
            result.set(attribute.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
