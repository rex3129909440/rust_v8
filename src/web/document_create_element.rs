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
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createElement' on 'Document': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'createElement' on 'Document'",
    ) else {
        return;
    };
    if !super::document::valid_xml_name(&name) {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            &format!(
                "Failed to execute 'createElement' on 'Document': The tag name provided ('{name}') is not a valid name."
            ),
        );
        return;
    }
    let element = if super::document::content_type(scope, arguments.this()) == Some("text/html") {
        super::document::create_html_element_by_name(scope, &name.to_ascii_lowercase())
    } else {
        super::element::create(scope, name.clone(), None)
    };
    match element {
        Ok(element) => {
            super::node::set_owner_document(scope, element, arguments.this());
            if arguments.length() > 1
                && let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(1))
                && let Some(key) = v8::String::new(scope, "is")
                && let Some(value) = options.get(scope, key.into())
                && !value.is_undefined()
            {
                let Some(is_name) = crate::webidl::dom_string_with_context(
                    scope,
                    value,
                    "Failed to execute 'createElement' on 'Document': Failed to read the 'is' property from 'ElementCreationOptions'",
                ) else {
                    return;
                };
                super::custom_element_registry::set_candidate_is(scope, element, Some(is_name));
            }
            super::custom_element_registry::try_construct_created(scope, element);
            if super::custom_element_registry::is_failed(scope, element)
                && super::document::content_type(scope, arguments.this()) == Some("text/html")
            {
                match super::html_unknown_element::create(scope, &name.to_ascii_lowercase()) {
                    Ok(fallback) => {
                        super::node::set_owner_document(scope, fallback, arguments.this());
                        result.set(fallback.into());
                    }
                    Err(message) => crate::webidl::throw_type_error(scope, &message),
                }
            } else {
                result.set(element.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
