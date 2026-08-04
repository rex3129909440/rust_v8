pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "children", get_children)
}

fn get_children(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(collection) = super::element::cached_children_collection(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &collection).into());
        return;
    }
    match super::html_collection::create_live(
        scope,
        arguments.this(),
        super::html_collection::HtmlCollectionQuery::Children,
    ) {
        Ok(collection) => {
            super::element::cache_children_collection(scope, arguments.this(), collection);
            result.set(collection.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
