pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "nodeValue", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::node::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value =
        super::character_data::data_if_character(scope, arguments.this()).or(record.node_value);
    match value {
        Some(value) => {
            if let Some(value) = v8::String::new(scope, &value) {
                result.set(value.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = super::node::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = if arguments.get(0).is_null() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    if super::character_data::set_data_if_character(scope, arguments.this(), value.clone()) {
        return;
    }
    if record.node_value.is_some() {
        super::node::set_stored_node_value(scope, arguments.this(), Some(value));
    }
}
