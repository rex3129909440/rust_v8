pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "serializeToString",
        1,
        serialize_to_string,
    )
}

fn serialize_to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::xml_serializer::is_instance(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'serializeToString' on 'XMLSerializer': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'serializeToString' on 'XMLSerializer': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'serializeToString' on 'XMLSerializer': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    match super::dom_html::serialize_xml_node(scope, node) {
        Ok(serialized) => {
            if let Some(serialized) = v8::String::new(scope, &serialized) {
                result.set(serialized.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
