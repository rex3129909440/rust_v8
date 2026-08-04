use super::html_directory_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "compact", get_compact, set_compact)
}

fn get_compact(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(compact) = scope
        .get_slot::<HtmlDirectoryElementStore>()
        .and_then(|store| {
            store
                .compact
                .get(&arguments.this().get_identity_hash().get())
        })
        .copied()
    {
        result.set(v8::Boolean::new(scope, compact).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_compact(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(compact) = scope
        .get_slot_mut::<HtmlDirectoryElementStore>()
        .and_then(|store| {
            store
                .compact
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *compact = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
