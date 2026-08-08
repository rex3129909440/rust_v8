pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "elementsFromPoint",
        2,
        elements_from_point,
    )
}
fn elements_from_point(
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
    let mut values =
        super::document_method_support::hit_test_elements_in_root(scope, a.this(), x, y);
    if let Some(host) = super::shadow_root::host(scope, a.this())
        && super::document_method_support::hit_test_element(scope, host, x, y)
    {
        values.push(host);
    }
    let out = v8::Array::new(scope, values.len() as i32);
    for (i, v) in values.into_iter().enumerate() {
        let _ = out.set_index(scope, i as u32, v.into());
    }
    r.set(out.into())
}
