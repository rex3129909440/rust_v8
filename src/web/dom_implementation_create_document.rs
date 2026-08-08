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
    let namespace = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        (!value.is_empty()).then_some(value)
    };
    let qualified_name = crate::webidl::value_to_string(scope, arguments.get(1));
    if !qualified_name.is_empty()
        && let Err((name, message)) =
            super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, false)
    {
        super::node::throw_dom_exception(scope, name, message);
        return;
    }
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
            crate::webidl::throw_type_error(scope, "The doctype is not a DocumentType");
            return;
        };
        if !super::node::record(scope, doctype).is_some_and(|record| record.node_type == 10) {
            crate::webidl::throw_type_error(scope, "The doctype is not a DocumentType");
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
