pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "elementFromPoint", 2, element_from_point)
}
fn element_from_point(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let x = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    let y = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    let value = super::document_method_support::hit_test_elements_in_root(scope, a.this(), x, y)
        .into_iter()
        .next()
        .or_else(|| {
            super::shadow_root::host(scope, a.this())
                .filter(|host| super::document_method_support::hit_test_element(scope, *host, x, y))
        });
    if let Some(v) = value {
        r.set(v.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
