pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "replaceData", 3, replace_data)
}

fn replace_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::character_data::data_if_character(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if !super::character_data::require_arguments(scope, &arguments, "replaceData", 3) {
        return;
    }
    let Some(offset) =
        super::character_data::unsigned_long_argument(scope, arguments.get(0), "replaceData")
    else {
        return;
    };
    let Some(count) =
        super::character_data::unsigned_long_argument(scope, arguments.get(1), "replaceData")
    else {
        return;
    };
    let Some(value) = super::character_data::string_argument(
        scope,
        arguments.get(2),
        "Failed to execute 'replaceData' on 'CharacterData'",
    ) else {
        return;
    };
    match super::character_data::replace_data_units(scope, arguments.this(), offset, count, &value)
    {
        Ok(()) => {}
        Err(super::character_data::EditError::IllegalInvocation) => {
            crate::webidl::throw_type_error(scope, "Illegal invocation")
        }
        Err(super::character_data::EditError::IndexSize) => {
            super::character_data::throw_offset_error(
                scope,
                arguments.this(),
                "replaceData",
                offset,
            )
        }
    }
}
