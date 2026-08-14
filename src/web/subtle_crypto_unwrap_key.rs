pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "unwrapKey", 7, unwrap_key)
}

fn unwrap_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments, "unwrapKey", &mut result)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 7, "unwrapKey")
    {
        return;
    }
    let format = crate::webidl::value_to_string(scope, arguments.get(0));
    if format != "raw" {
        super::subtle_crypto_support::reject_not_supported(
            scope,
            &mut result,
            "Only raw key unwrapping is currently supported",
        );
        return;
    }
    let Some(wrapped_key) = super::subtle_crypto_support::bytes(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "wrappedKey must be a BufferSource");
        return;
    };
    let Some(unwrapping_key) = super::crypto_key::record_from_value(scope, arguments.get(2)) else {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The unwrappingKey argument is not a CryptoKey",
        );
        return;
    };
    if let Err(message) =
        super::subtle_crypto_support::require_key_usage(&unwrapping_key, "unwrapKey")
    {
        super::subtle_crypto_support::reject(scope, &mut result, "InvalidAccessError", &message);
        return;
    }
    let (unwrap_name, parameters) =
        match super::subtle_crypto_support::cipher_parameters(scope, arguments.get(3)) {
            Ok(parameters) => parameters,
            Err(message) => {
                super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
                return;
            }
        };
    if unwrapping_key.algorithm.name() != unwrap_name {
        super::subtle_crypto_support::reject(
            scope,
            &mut result,
            "InvalidAccessError",
            "The unwrap algorithm does not match the unwrapping CryptoKey",
        );
        return;
    }
    let material = match super::subtle_crypto_support::decrypt_with_parameters(
        &unwrapping_key.material,
        &parameters,
        &wrapped_key,
    ) {
        Ok(material) => material,
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "OperationError", &message);
            return;
        }
    };
    let unwrapped_algorithm_value = arguments.get(4);
    let unwrapped_name =
        match super::subtle_crypto_support::algorithm_name(scope, unwrapped_algorithm_value) {
            Ok(name) => name,
            Err(message) => {
                super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
                return;
            }
        };
    let extractable = arguments.get(5).boolean_value(scope);
    let usages = match super::subtle_crypto_support::usages(scope, arguments.get(6)) {
        Ok(usages) => usages,
        Err(message) => {
            super::subtle_crypto_support::reject(scope, &mut result, "SyntaxError", &message);
            return;
        }
    };
    let algorithm = match super::subtle_crypto_import_key::imported_algorithm(
        scope,
        unwrapped_algorithm_value,
        &unwrapped_name,
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
