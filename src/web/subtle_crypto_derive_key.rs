pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "deriveKey", 5, derive_key)
}

fn derive_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 5, "deriveKey")
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
    if let Err(message) = super::subtle_crypto_support::require_key_usage(&base_key, "deriveKey") {
        super::subtle_crypto_support::reject(scope, &mut result, "InvalidAccessError", &message);
        return;
    }
    let derived_algorithm_value = arguments.get(2);
    let name = match super::subtle_crypto_support::algorithm_name(scope, derived_algorithm_value) {
        Ok(name) => name,
        Err(message) => {
            super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
            return;
        }
    };
    let object = match v8::Local::<v8::Object>::try_from(derived_algorithm_value) {
        Ok(object) => object,
        Err(_) => {
            super::subtle_crypto_support::reject(
                scope,
                &mut result,
                "DataError",
                "The derived key algorithm must be an object",
            );
            return;
        }
    };
    let extractable = arguments.get(3).boolean_value(scope);
    let usages = match super::subtle_crypto_support::usages(scope, arguments.get(4)) {
        Ok(usages) => usages,
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "SyntaxError", &message);
            return;
        }
    };
    if usages.is_empty() {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "SyntaxError",
            "A derived secret key must have at least one usage",
        );
        return;
    }
    let (algorithm, length_bits) = match name.as_str() {
        "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => {
            let allowed = if name == "AES-KW" {
                &["wrapKey", "unwrapKey"][..]
            } else {
                &["encrypt", "decrypt", "wrapKey", "unwrapKey"][..]
            };
            if let Err(message) = super::subtle_crypto_support::validate_usages(&usages, allowed) {
                super::subtle_crypto_support::reject(scope, &mut result, "SyntaxError", &message);
                return;
            }
            let length =
                match super::subtle_crypto_support::number_property(scope, object, "length") {
                    Ok(length)
                        if length.fract() == 0.0 && matches!(length as u32, 128 | 192 | 256) =>
                    {
                        length as u16
                    }
                    _ => {
                        super::subtle_crypto_support::reject(
                            scope,
                            &mut result,
                            "OperationError",
                            "AES length must be 128, 192, or 256",
                        );
                        return;
                    }
                };
            (
                super::crypto_key::KeyAlgorithm::Aes {
                    name: name.clone(),
                    length,
                },
                u32::from(length),
            )
        }
        "HMAC" => {
            if let Err(message) =
                super::subtle_crypto_support::validate_usages(&usages, &["sign", "verify"])
            {
                super::subtle_crypto_support::reject(scope, &mut result, "SyntaxError", &message);
                return;
            }
            let hash =
                match super::subtle_crypto_support::hash_property(scope, derived_algorithm_value) {
                    Ok(hash) => hash,
                    Err(message) => {
                        super::subtle_crypto_support::reject_not_supported(
                            scope,
                            &mut result,
                            &message,
                        );
                        return;
                    }
                };
            let length =
                super::subtle_crypto_support::optional_number_property(scope, object, "length")
                    .unwrap_or((hash.block_bytes() * 8) as f64);
            if !length.is_finite()
                || length <= 0.0
                || length.fract() != 0.0
                || length % 8.0 != 0.0
                || length > u32::MAX as f64
            {
                super::subtle_crypto_support::reject(
                    scope,
                    &mut result,
                    "OperationError",
                    "HMAC length must be a positive byte-aligned integer",
                );
                return;
            }
            (
                super::crypto_key::KeyAlgorithm::Hmac {
                    hash: hash.name().to_owned(),
                    length: length as u32,
                },
                length as u32,
            )
        }
        _ => {
            super::subtle_crypto_support::reject_not_supported(
                scope,
                &mut result,
                &format!("Deriving a '{name}' key is not supported"),
            );
            return;
        }
    };
    let material = match super::subtle_crypto_support::derive_bits_material(
        scope,
        arguments.get(0),
        &base_key,
        length_bits,
    ) {
        Ok(material) => material,
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "OperationError", &message);
            return;
        }
    };
    let key =
        match super::crypto_key::create_secret(scope, extractable, algorithm, usages, material) {
            Ok(key) => key,
            Err(message) => {
                super::subtle_crypto_support::reject(
                    scope,
                    &mut result,
                    "OperationError",
                    &message,
                );
                return;
            }
        };
    super::subtle_crypto_support::resolve_value(scope, &mut result, key.into());
}
