pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "sign", 3, sign)
}

fn sign(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments, "sign", &mut result)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 3, "sign")
    {
        return;
    }
    let name = match super::subtle_crypto_support::algorithm_name(scope, arguments.get(0)) {
        Ok(name) => name,
        Err(message) => {
            super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
            return;
        }
    };
    let Some(key) = super::crypto_key::record_from_value(scope, arguments.get(1)) else {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The key argument is not a CryptoKey",
        );
        return;
    };
    if name != key.algorithm.name() {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The requested algorithm does not match the CryptoKey",
        );
        return;
    }
    if let Err(message) = super::subtle_crypto_support::require_key_usage(&key, "sign") {
        super::subtle_crypto_support::reject(scope, &mut result, "InvalidAccessError", &message);
        return;
    }
    let Some(data) = super::subtle_crypto_support::bytes(arguments.get(2)) else {
        crate::webidl::throw_type_error(scope, "The data argument must be a BufferSource");
        return;
    };
    let super::crypto_key::KeyAlgorithm::Hmac { hash, .. } = &key.algorithm else {
        super::subtle_crypto_support::reject_not_supported(
            scope,
            &mut result,
            &format!("Signing with '{name}' is not supported"),
        );
        return;
    };
    let hash = match hash.as_str() {
        "SHA-1" => super::subtle_crypto_support::HashAlgorithm::Sha1,
        "SHA-256" => super::subtle_crypto_support::HashAlgorithm::Sha256,
        "SHA-384" => super::subtle_crypto_support::HashAlgorithm::Sha384,
        "SHA-512" => super::subtle_crypto_support::HashAlgorithm::Sha512,
        _ => {
            super::subtle_crypto_support::reject_not_supported(
                scope,
                &mut result,
                "The HMAC key uses an unsupported hash",
            );
            return;
        }
    };
    match super::subtle_crypto_support::hmac(hash, &key.material, &data) {
        Ok(signature) => {
            super::subtle_crypto_support::resolve_array_buffer(scope, &mut result, signature);
        }
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "OperationError", &message);
        }
    }
}
