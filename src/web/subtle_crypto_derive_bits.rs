pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "deriveBits", 2, derive_bits)
}

fn derive_bits(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments, "deriveBits", &mut result)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 2, "deriveBits")
    {
        return;
    }
    let Some(base_key) = super::crypto_key::record_from_value(scope, arguments.get(1)) else {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The baseKey argument is not a CryptoKey",
        );
        return;
    };
    if let Err(message) = super::subtle_crypto_support::require_key_usage(&base_key, "deriveBits") {
        super::subtle_crypto_support::reject(scope, &mut result, "InvalidAccessError", &message);
        return;
    }
    let Some(length) = arguments.get(2).number_value(scope) else {
        crate::webidl::throw_type_error(scope, "The length argument must be a number");
        return;
    };
    if !length.is_finite() || length <= 0.0 || length.fract() != 0.0 || length > u32::MAX as f64 {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "OperationError",
            "The derived bit length is invalid",
        );
        return;
    }
    match super::subtle_crypto_support::derive_bits_material(
        scope,
        arguments.get(0),
        &base_key,
        length as u32,
    ) {
        Ok(bits) => {
            super::subtle_crypto_support::resolve_array_buffer(scope, &mut result, bits);
        }
        Err(message) => {
            let error = if message.contains("does not match") {
                "InvalidAccessError"
            } else {
                "OperationError"
            };
            super::subtle_crypto_support::reject(scope, &mut result, error, &message);
        }
    }
}
