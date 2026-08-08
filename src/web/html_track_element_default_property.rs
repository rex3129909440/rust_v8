use super::html_track_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "default", get_default, set_default)
}

fn get_default(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.default_enabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_default(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTrackElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.default_enabled = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
