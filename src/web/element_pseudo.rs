pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "pseudo", 1, pseudo)
}

fn pseudo(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'pseudo' on 'Element': 1 argument required, but only 0 present.",
        );
        return;
    }
    let pseudo_type = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::css_pseudo_element::valid_type(&pseudo_type) {
        result.set(v8::null(scope).into());
        return;
    }
    match super::css_pseudo_element::create(scope, pseudo_type, arguments.this(), arguments.this())
    {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
