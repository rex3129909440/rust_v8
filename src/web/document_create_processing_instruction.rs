pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createProcessingInstruction",
        2,
        create_processing_instruction,
    )
}

fn create_processing_instruction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let target = crate::webidl::value_to_string(scope, arguments.get(0));
    let data = crate::webidl::value_to_string(scope, arguments.get(1));
    if !super::document::valid_xml_name(&target) || target.contains(':') {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "The target is not a valid XML name",
        );
        return;
    }
    if data.contains("?>") {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            "Processing instruction data cannot contain '?>'",
        );
        return;
    }
    match super::processing_instruction::create(scope, target, data) {
        Ok(instruction) => {
            super::node::set_owner_document(scope, instruction, arguments.this());
            result.set(instruction.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
