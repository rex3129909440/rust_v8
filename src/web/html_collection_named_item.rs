pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)
}
fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    super::html_collection::refresh_live(scope, arguments.this());
    let Some(record) = super::html_collection::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'namedItem' on 'HTMLCollection': 1 argument required, but only 0 present.",
        );
        return;
    }
    if arguments.get(0).is_symbol() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'namedItem' on 'HTMLCollection': Cannot convert a Symbol value to a string",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(item) = super::html_collection::named_match(scope, record, &name) {
        result.set(item.into());
        return;
    }
    result.set(v8::null(scope).into());
}
