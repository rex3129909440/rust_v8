pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getElementsByTagName",
        1,
        get_elements_by_tag_name,
    )
}

fn get_elements_by_tag_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let tag_name = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::html_collection::create_live(
        scope,
        arguments.this(),
        super::html_collection::HtmlCollectionQuery::TagName(tag_name),
    ) {
        Ok(collection) => result.set(collection.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
