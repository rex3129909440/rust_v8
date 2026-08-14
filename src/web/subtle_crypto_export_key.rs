pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "exportKey", 2, export_key)
}

fn export_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments, "exportKey", &mut result)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 2, "exportKey")
    {
        return;
    }
    let format = crate::webidl::value_to_string(scope, arguments.get(0));
    if format != "raw" {
        super::subtle_crypto_support::reject_not_supported(
            scope,
            &mut result,
            "Only raw secret-key export is currently supported",
        );
        return;
    }
    let Some(key) = super::crypto_key::record_from_value(scope, arguments.get(1)) else {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The key argument is not a CryptoKey",
        );
        return;
    };
    if !key.extractable {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The CryptoKey is not extractable",
        );
        return;
    }
    super::subtle_crypto_support::resolve_array_buffer(scope, &mut result, key.material);
}
