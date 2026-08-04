pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "decrypt", 3, decrypt)
}

fn decrypt(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 3, "decrypt")
    {
        return;
    }
    let (name, parameters) =
        match super::subtle_crypto_support::cipher_parameters(scope, arguments.get(0)) {
            Ok(parameters) => parameters,
            Err(message) => {
                super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
                return;
            }
        };
    if name == "AES-KW" {
        super::subtle_crypto_support::reject_not_supported(
            scope,
            &mut result,
            "AES-KW is available through unwrapKey, not decrypt",
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
    if key.algorithm.name() != name {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The requested algorithm does not match the CryptoKey",
        );
        return;
    }
    if let Err(message) = super::subtle_crypto_support::require_key_usage(&key, "decrypt") {
        super::subtle_crypto_support::reject(scope, &mut result, "InvalidAccessError", &message);
        return;
    }
    let Some(ciphertext) = super::subtle_crypto_support::bytes(arguments.get(2)) else {
        crate::webidl::throw_type_error(scope, "The data argument must be a BufferSource");
        return;
    };
    match super::subtle_crypto_support::decrypt_with_parameters(
        &key.material,
        &parameters,
        &ciphertext,
    ) {
        Ok(plaintext) => {
            super::subtle_crypto_support::resolve_array_buffer(scope, &mut result, plaintext);
        }
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "OperationError", &message);
        }
    }
}
