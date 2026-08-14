pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "digest", 2, digest)
}

fn digest(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::subtle_crypto_support::require_receiver(scope, &arguments, "digest", &mut result)
        || !super::subtle_crypto_support::require_arguments(scope, &arguments, 2, "digest")
    {
        return;
    }
    let hash = match super::subtle_crypto_support::hash_algorithm(scope, arguments.get(0)) {
        Ok(hash) => hash,
        Err(message) => {
            super::subtle_crypto_support::reject_not_supported(scope, &mut result, &message);
            return;
        }
    };
    let Some(data) = super::subtle_crypto_support::bytes(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The data argument must be a BufferSource");
        return;
    };
    let output = super::subtle_crypto_support::digest(hash, &data);
    super::subtle_crypto_support::resolve_array_buffer(scope, &mut result, output);
}
