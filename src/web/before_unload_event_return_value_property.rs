use super::before_unload_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "returnValue",
        get_return_value,
        set_return_value,
    )
}

fn get_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<BeforeUnloadEventStore>()
        .and_then(|store| {
            store
                .values
                .get(&arguments.this().get_identity_hash().get())
        })
        && let Some(value) = v8::String::new(scope, value)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(current) = scope
        .get_slot_mut::<BeforeUnloadEventStore>()
        .and_then(|store| {
            store
                .values
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
