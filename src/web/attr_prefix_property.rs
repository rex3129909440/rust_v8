pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "prefix", get_prefix)
}

fn get_prefix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::attr::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.prefix {
        Some(prefix) => {
            if let Some(value) = v8::String::new(scope, &prefix) {
                result.set(value.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
