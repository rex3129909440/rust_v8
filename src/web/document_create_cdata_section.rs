pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createCDATASection",
        1,
        create_cdata_section,
    )
}

fn create_cdata_section(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if super::document::content_type(scope, arguments.this()) == Some("text/html") {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "CDATA sections are not supported by HTML documents",
        );
        return;
    }
    let data = crate::webidl::value_to_string(scope, arguments.get(0));
    if data.contains("]]>") {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "CDATA section data cannot contain ']]>'",
        );
        return;
    }
    match super::cdata_section::create(scope, data) {
        Ok(section) => {
            super::node::set_owner_document(scope, section, arguments.this());
            result.set(section.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
