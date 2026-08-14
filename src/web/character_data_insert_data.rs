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
    if super::character_data::data_if_character(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if !super::character_data::require_arguments(scope, &arguments, "insertData", 2) {
        return;
    }
    let Some(offset) =
        super::character_data::unsigned_long_argument(scope, arguments.get(0), "insertData")
    else {
        return;
    };
    let Some(value) = super::character_data::string_argument(
        scope,
        arguments.get(1),
        "Failed to execute 'insertData' on 'CharacterData'",
    ) else {
        return;
    };
    match super::character_data::replace_data_units(scope, arguments.this(), offset, 0, &value) {
        Ok(()) => {}
        Err(super::character_data::EditError::IllegalInvocation) => {
            crate::webidl::throw_type_error(scope, "Illegal invocation")
        }
        Err(super::character_data::EditError::IndexSize) => {
            super::character_data::throw_offset_error(scope, arguments.this(), "insertData", offset)
        }
    }
}
