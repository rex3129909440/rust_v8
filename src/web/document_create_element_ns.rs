pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createElementNS", 2, create_element_ns)
}

fn create_element_ns(
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
                "Failed to execute 'createElementNS' on 'Document': 2 arguments required, but only {} present.",
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
            "Failed to execute 'createElementNS' on 'Document'",
        ) else {
            return;
        };
        (!namespace.is_empty()).then_some(namespace)
    };
    let Some(qualified_name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(1),
        "Failed to execute 'createElementNS' on 'Document'",
    ) else {
        return;
    };
    if let Err((name, _)) =
        super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, false)
    {
        let message = super::document::qualified_name_error_message(
            "createElementNS",
            "Document",
            name,
            namespace.as_deref(),
            &qualified_name,
        );
        super::node::throw_dom_exception(scope, name, &message);
        return;
    }
    let qualified_name = super::document::canonical_qualified_name(&qualified_name);
    let local_name = qualified_name
        .rsplit_once(':')
        .map(|(_, local_name)| local_name)
        .unwrap_or(&qualified_name);
    let created = match namespace.as_deref() {
        Some("http://www.w3.org/2000/svg") => {
            super::document::create_svg_element(scope, local_name)
        }
        Some("http://www.w3.org/1998/Math/MathML") => {
            super::math_ml_element::create(scope, local_name.to_owned())
        }
        Some("http://www.w3.org/1999/xhtml") if local_name == local_name.to_ascii_lowercase() => {
            super::document::create_html_element_by_name(scope, local_name)
        }
        Some("http://www.w3.org/1999/xhtml") => {
            super::html_unknown_element::create(scope, &qualified_name)
        }
        _ => super::element::create(scope, qualified_name.clone(), namespace.clone()),
    };
    match created {
        Ok(element) => {
            super::element::set_qualified_name(scope, element, qualified_name);
            super::node::set_owner_document(scope, element, arguments.this());
            result.set(element.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
