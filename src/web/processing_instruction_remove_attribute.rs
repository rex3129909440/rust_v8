pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "removeAttribute", 1, remove_attribute)
}

fn remove_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(mut attributes) = super::processing_instruction::attributes(scope, arguments.this())
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'removeAttribute' on 'ProcessingInstruction': 1 argument required",
        );
        return;
    }
    let name = super::processing_instruction::requested_name(scope, &arguments);
    attributes.retain(|(candidate, _)| candidate != &name);
    super::processing_instruction::write_attributes(scope, arguments.this(), &attributes);
}
