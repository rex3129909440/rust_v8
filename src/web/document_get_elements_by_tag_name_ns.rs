pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getElementsByTagNameNS",
        2,
        get_elements_by_tag_name_ns,
    )
}

fn get_elements_by_tag_name_ns(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let namespace = if arguments.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, arguments.get(0)))
    };
    let local_name = crate::webidl::value_to_string(scope, arguments.get(1));
    match super::html_collection::create_live(
        scope,
        arguments.this(),
        super::html_collection::HtmlCollectionQuery::TagNameNs {
            namespace,
            local_name,
        },
    ) {
        Ok(collection) => result.set(collection.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
