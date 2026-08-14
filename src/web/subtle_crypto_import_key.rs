pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "importKey", 5, import_key)
}

fn import_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments, "importKey", &mut result)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 5, "importKey")
    {
        return;
    }
    let format = crate::webidl::value_to_string(scope, arguments.get(0));
    if format != "raw" {
        super::subtle_crypto_support::reject_not_supported(
            scope,
            &mut result,
            "Only raw secret-key import is currently supported",
        );
        return;
    }
    let Some(material) = super::subtle_crypto_support::bytes(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "keyData must be a BufferSource for raw import");
        return;
    };
    let algorithm_value = arguments.get(2);
    let algorithm_name = match super::subtle_crypto_support::algorithm_name(scope, algorithm_value)
    {
        Ok(name) => name,
        Err(message) => {
            super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
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
    let algorithm = match imported_algorithm(
        scope,
        algorithm_value,
        &algorithm_name,
        material.len(),
        extractable,
        &usages,
    ) {
        Ok(algorithm) => algorithm,
        Err((name, message)) => {
            super::subtle_crypto_support::reject(scope, &mut result, name, &message);
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

pub(crate) fn imported_algorithm(
    scope: &mut v8::PinScope<'_, '_>,
    algorithm_value: v8::Local<'_, v8::Value>,
    algorithm_name: &str,
    material_length: usize,
    extractable: bool,
    usages: &[String],
) -> Result<super::crypto_key::KeyAlgorithm, (&'static str, String)> {
    match algorithm_name {
        "HMAC" => {
            if material_length == 0 {
                return Err((
                    "DataError",
                    "HMAC key material must not be empty".to_owned(),
                ));
            }
            super::subtle_crypto_support::validate_usages(usages, &["sign", "verify"])
                .map_err(|message| ("SyntaxError", message))?;
            let hash = super::subtle_crypto_support::hash_property(scope, algorithm_value)
                .map_err(|message| ("NotSupportedError", message))?;
            let object = v8::Local::<v8::Object>::try_from(algorithm_value)
                .map_err(|_| ("DataError", "HMAC parameters must be an object".to_owned()))?;
            let supplied_length =
                super::subtle_crypto_support::optional_number_property(scope, object, "length");
            let material_bits = (material_length as u32).saturating_mul(8);
            let length = supplied_length.map_or(Ok(material_bits), |value| {
                if value.is_finite()
                    && value > 0.0
                    && value.fract() == 0.0
                    && value <= material_bits as f64
                    && value % 8.0 == 0.0
                {
                    Ok(value as u32)
                } else {
                    Err((
                        "DataError",
                        "HMAC length must be a positive byte-aligned value no larger than the key"
                            .to_owned(),
                    ))
                }
            })?;
            Ok(super::crypto_key::KeyAlgorithm::Hmac {
                hash: hash.name().to_owned(),
                length,
            })
        }
        "AES-CBC" | "AES-CTR" | "AES-GCM" | "AES-KW" => {
            let allowed = if algorithm_name == "AES-KW" {
                &["wrapKey", "unwrapKey"][..]
            } else {
                &["encrypt", "decrypt", "wrapKey", "unwrapKey"][..]
            };
            super::subtle_crypto_support::validate_usages(usages, allowed)
                .map_err(|message| ("SyntaxError", message))?;
            if !matches!(material_length, 16 | 24 | 32) {
                return Err((
                    "DataError",
                    "AES key material must be 128, 192, or 256 bits".to_owned(),
                ));
            }
            Ok(super::crypto_key::KeyAlgorithm::Aes {
                name: algorithm_name.to_owned(),
                length: (material_length * 8) as u16,
            })
        }
        "PBKDF2" => {
            if extractable {
                return Err((
                    "SyntaxError",
                    "PBKDF2 keys must be non-extractable".to_owned(),
                ));
            }
            super::subtle_crypto_support::validate_usages(usages, &["deriveBits", "deriveKey"])
                .map_err(|message| ("SyntaxError", message))?;
            Ok(super::crypto_key::KeyAlgorithm::Pbkdf2)
        }
        "HKDF" => {
            if extractable {
                return Err((
                    "SyntaxError",
                    "HKDF keys must be non-extractable".to_owned(),
                ));
            }
            super::subtle_crypto_support::validate_usages(usages, &["deriveBits", "deriveKey"])
                .map_err(|message| ("SyntaxError", message))?;
            Ok(super::crypto_key::KeyAlgorithm::Hkdf)
        }
        _ => Err((
            "NotSupportedError",
            format!("The key algorithm '{algorithm_name}' is not supported"),
        )),
    }
}
