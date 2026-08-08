pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setAttribute", 2, set_attribute)
}

fn set_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(mut attributes) = super::processing_instruction::attributes(scope, arguments.this())
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'setAttribute' on 'ProcessingInstruction': 2 arguments required",
        );
        return;
    }
    let name = super::processing_instruction::requested_name(scope, &arguments);
    if !super::processing_instruction::validate_attribute_name(scope, &name) {
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    if let Some((_, stored)) = attributes
        .iter_mut()
        .find(|(candidate, _)| candidate == &name)
    {
        *stored = value;
    } else {
        attributes.push((name, value));
    }
    super::processing_instruction::write_attributes(scope, arguments.this(), &attributes);
}
