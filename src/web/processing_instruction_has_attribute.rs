pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "hasAttribute", 1, has_attribute)
}

fn has_attribute(
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
            "Failed to execute 'hasAttribute' on 'ProcessingInstruction': 1 argument required",
        );
        return;
    }
    let name = super::processing_instruction::requested_name(scope, &arguments);
    result.set(
        v8::Boolean::new(
            scope,
            attributes.iter().any(|(candidate, _)| candidate == &name),
        )
        .into(),
    );
}
