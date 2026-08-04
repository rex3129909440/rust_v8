pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toggleAttribute", 1, toggle_attribute)
}

fn toggle_attribute(
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
            "Failed to execute 'toggleAttribute' on 'ProcessingInstruction': 1 argument required",
        );
        return;
    }
    let name = super::processing_instruction::requested_name(scope, &arguments);
    if !super::processing_instruction::validate_attribute_name(scope, &name) {
        return;
    }
    let existing = attributes
        .iter()
        .position(|(candidate, _)| candidate == &name);
    let should_exist = if arguments.length() >= 2 {
        arguments.get(1).boolean_value(scope)
    } else {
        existing.is_none()
    };
    match (existing, should_exist) {
        (Some(index), false) => {
            attributes.remove(index);
        }
        (None, true) => attributes.push((name, String::new())),
        _ => {}
    }
    super::processing_instruction::write_attributes(scope, arguments.this(), &attributes);
}
