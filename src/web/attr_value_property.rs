pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::attr::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.value) {
        result.set(value.into());
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = super::attr::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(owner) = snapshot.owner_element {
        super::element::set_attribute_from_attr(
            scope,
            v8::Local::new(scope, &owner),
            snapshot.name,
            value.clone(),
            snapshot.namespace_uri,
        );
    }
    super::attr::set_stored_value(scope, arguments.this(), value);
}
