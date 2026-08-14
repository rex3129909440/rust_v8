use super::dom_implementation::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createDocumentType",
        3,
        create_document_type,
    )
}

fn create_document_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 3 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'createDocumentType' on 'DOMImplementation': 3 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let Some(qualified_name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'createDocumentType' on 'DOMImplementation'",
    ) else {
        return;
    };
    if !super::document::valid_xml_name(&qualified_name)
        || qualified_name.matches(':').count() > 1
        || qualified_name.starts_with(':')
        || qualified_name.ends_with(':')
    {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            &format!(
                "Failed to execute 'createDocumentType' on 'DOMImplementation': The qualified name provided ('{qualified_name}') is not a valid name."
            ),
        );
        return;
    }
    let Some(public_id) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(1),
        "Failed to execute 'createDocumentType' on 'DOMImplementation'",
    ) else {
        return;
    };
    let Some(system_id) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(2),
        "Failed to execute 'createDocumentType' on 'DOMImplementation'",
    ) else {
        return;
    };
    match super::document_type::create(scope, &qualified_name, &public_id, &system_id) {
        Ok(doctype) => {
            if let Some(document) =
                super::dom_implementation::associated_document(scope, arguments.this())
            {
                super::node::set_owner_document(scope, doctype, document);
            }
            result.set(doctype.into())
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
