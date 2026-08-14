use super::dom_implementation::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createDocument", 2, create_document)
}

fn create_document(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'createDocument' on 'DOMImplementation': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let namespace = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        let Some(value) = crate::webidl::dom_string_with_context(
            scope,
            arguments.get(0),
            "Failed to execute 'createDocument' on 'DOMImplementation'",
        ) else {
            return;
        };
        (!value.is_empty()).then_some(value)
    };
    let Some(qualified_name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(1),
        "Failed to execute 'createDocument' on 'DOMImplementation'",
    ) else {
        return;
    };
    if !qualified_name.is_empty()
        && let Err((name, _)) =
            super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, false)
    {
        let message = super::document::qualified_name_error_message(
            "createDocument",
            "DOMImplementation",
            name,
            namespace.as_deref(),
            &qualified_name,
        );
        super::node::throw_dom_exception(scope, name, &message);
        return;
    }
    let qualified_name = super::document::canonical_qualified_name(&qualified_name);
    let document =
        match super::xml_document::create_with_type(scope, String::new(), "application/xml") {
            Ok(document) => document,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
    if !arguments.get(2).is_null_or_undefined() {
        let Ok(doctype) = v8::Local::<v8::Object>::try_from(arguments.get(2)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'createDocument' on 'DOMImplementation': parameter 3 is not of type 'DocumentType'.",
            );
            return;
        };
        if !super::node::record(scope, doctype).is_some_and(|record| record.node_type == 10) {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'createDocument' on 'DOMImplementation': parameter 3 is not of type 'DocumentType'.",
            );
            return;
        }
        super::node::set_owner_document(scope, doctype, document);
        let length = super::node::children(scope, document).len();
        if let Err((name, message)) = super::node::insert_node(scope, document, doctype, length) {
            super::node::throw_dom_exception(scope, name, message);
            return;
        }
    }
    if !qualified_name.is_empty() {
        let element = match super::element::create(scope, qualified_name, namespace) {
            Ok(element) => element,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        super::node::set_owner_document(scope, element, document);
        let length = super::node::children(scope, document).len();
        if let Err((name, message)) = super::node::insert_node(scope, document, element, length) {
            super::node::throw_dom_exception(scope, name, message);
            return;
        }
    }
    result.set(document.into());
}
