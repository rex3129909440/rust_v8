pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "wrapKey", 4, wrap_key)
}

fn wrap_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 4, "wrapKey")
    {
        return;
    }
    let format = crate::webidl::value_to_string(scope, arguments.get(0));
    if format != "raw" {
        super::subtle_crypto_support::reject_not_supported(
            scope,
            &mut result,
            "Only raw key wrapping is currently supported",
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
            "The key to wrap is not extractable",
        );
        return;
    }
    let Some(wrapping_key) = super::crypto_key::record_from_value(scope, arguments.get(2)) else {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The wrappingKey argument is not a CryptoKey",
        );
        return;
    };
    if let Err(message) = super::subtle_crypto_support::require_key_usage(&wrapping_key, "wrapKey")
    {
        super::subtle_crypto_support::reject(scope, &mut result, "InvalidAccessError", &message);
        return;
    }
    let (name, parameters) =
        match super::subtle_crypto_support::cipher_parameters(scope, arguments.get(3)) {
            Ok(parameters) => parameters,
            Err(message) => {
                super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
                return;
            }
        };
    if wrapping_key.algorithm.name() != name {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The wrap algorithm does not match the wrapping CryptoKey",
        );
        return;
    }
    match super::subtle_crypto_support::encrypt_with_parameters(
        &wrapping_key.material,
        &parameters,
        &key.material,
    ) {
        Ok(wrapped) => {
            super::subtle_crypto_support::resolve_array_buffer(scope, &mut result, wrapped);
        }
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "OperationError", &message);
        }
    }
}
