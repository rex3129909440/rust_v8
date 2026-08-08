pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "insertData", 2, insert_data)
}

fn insert_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let offset = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    match super::character_data::replace_data_units(scope, arguments.this(), offset, 0, &value) {
        Ok(()) => {}
        Err(super::character_data::EditError::IllegalInvocation) => {
            crate::webidl::throw_type_error(scope, "Illegal invocation")
        }
        Err(super::character_data::EditError::IndexSize) => super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "The offset is larger than the data length",
        ),
    }
}
