use super::svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "nonce", get_nonce, set_nonce)
}

pub(crate) fn get_nonce(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.nonce, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_nonce(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<SvgElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.nonce = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
