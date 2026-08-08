use super::html_frame_set_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "cols", get_cols, set_cols)
}

fn get_cols(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = v8::String::new(scope, &record.cols) {
            r.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_cols(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlFrameSetElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.cols = value
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
