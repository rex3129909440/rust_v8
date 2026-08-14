pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getElementsByClassName",
        1,
        get_elements_by_class_name,
    )
}

fn get_elements_by_class_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let class_names = source.split_ascii_whitespace().map(str::to_owned).collect();
    match super::html_collection::create_live(
        scope,
        arguments.this(),
        super::html_collection::HtmlCollectionQuery::ClassNames {
            names: class_names,
            source,
        },
    ) {
        Ok(collection) => result.set(collection.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
