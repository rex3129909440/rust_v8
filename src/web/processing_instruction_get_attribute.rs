pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getAttribute", 1, get_attribute)
}

fn get_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(attributes) = super::processing_instruction::attributes(scope, arguments.this())
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'getAttribute' on 'ProcessingInstruction': 1 argument required",
        );
        return;
    }
    let name = super::processing_instruction::requested_name(scope, &arguments);
    if let Some(value) = attributes
        .iter()
        .find_map(|(candidate, value)| (candidate == &name).then_some(value))
        .and_then(|value| v8::String::new(scope, value))
    {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}
