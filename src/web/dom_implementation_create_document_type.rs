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
    let qualified_name = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::document::valid_xml_name(&qualified_name)
        || qualified_name.matches(':').count() > 1
        || qualified_name.starts_with(':')
        || qualified_name.ends_with(':')
    {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "The qualified name is not valid",
        );
        return;
    }
    let public_id = crate::webidl::value_to_string(scope, arguments.get(1));
    let system_id = crate::webidl::value_to_string(scope, arguments.get(2));
    match super::document_type::create(scope, &qualified_name, &public_id, &system_id) {
        Ok(doctype) => result.set(doctype.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
