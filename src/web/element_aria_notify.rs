pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "ariaNotify", 1, aria_notify)
}

fn aria_notify(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'ariaNotify' on 'Element': 1 argument required.",
        );
        return;
    }
    let _announcement = crate::webidl::value_to_string(scope, arguments.get(0));
    if !arguments.get(1).is_undefined()
        && v8::Local::<v8::Object>::try_from(arguments.get(1)).is_err()
    {
        crate::webidl::throw_type_error(scope, "ariaNotify options must be an object");
    }
}
