pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "baseURI", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let object = arguments.this();
    let document = if super::document::is_document(scope, object) {
        Some(object)
    } else {
        super::node::owner_document(scope, object)
    };
    let base_url = document
        .map(|document| super::document::base_url(scope, document))
        .unwrap_or_else(|| crate::page_init::base_url(scope));
    if let Some(value) = v8::String::new(scope, &base_url) {
        result.set(value.into());
    }
}
