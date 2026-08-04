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
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = super::html_collection::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    for item in record {
        let item = v8::Local::new(scope, item);
        let matched = super::element::record(scope, item).is_some_and(|element| {
            element.attributes.iter().any(|(attribute, value)| {
                (attribute.eq_ignore_ascii_case("id") || attribute.eq_ignore_ascii_case("name"))
                    && value == &name
            })
        });
        if matched {
            result.set(item.into());
            return;
        }
    }
    result.set(v8::null(scope).into());
}
