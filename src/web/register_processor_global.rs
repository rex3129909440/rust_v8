pub(crate) fn install(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "registerProcessor", 2, register_processor)
}

fn register_processor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(constructor) = v8::Local::<v8::Function>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "Processor constructor must be callable");
        return;
    };
    if let Err(message) = super::worklet::register_processor(scope, name, constructor) {
        crate::webidl::throw_type_error(scope, &message);
    }
}
