pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "elementFromPoint", 2, element_from_point)
}

fn element_from_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let x = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let y = arguments.get(1).number_value(scope).unwrap_or(f64::NAN);
    match super::document_method_support::hit_test_elements(scope, arguments.this(), x, y)
        .into_iter()
        .next()
    {
        Some(element) => result.set(element.into()),
        None => result.set(v8::null(scope).into()),
    }
}
