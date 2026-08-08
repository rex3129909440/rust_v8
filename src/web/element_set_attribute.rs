pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setAttribute", 2, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = super::element::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut name = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::document::valid_xml_name(&name) {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "The attribute name is not valid",
        );
        return;
    }
    if record.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml") {
        name.make_ascii_lowercase();
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    super::element::set_attribute_value(scope, arguments.this(), name, value);
}
