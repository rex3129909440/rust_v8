pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "localName", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let local = record
        .tag_name
        .rsplit(':')
        .next()
        .unwrap_or(&record.tag_name);
    let local = if record.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
        local.to_ascii_lowercase()
    } else {
        local.to_owned()
    };
    if let Some(local) = v8::String::new(scope, &local) {
        result.set(local.into());
    }
}
