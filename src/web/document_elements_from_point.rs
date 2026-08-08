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
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let x = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let y = arguments.get(1).number_value(scope).unwrap_or(f64::NAN);
    let values = super::document_method_support::hit_test_elements(scope, arguments.this(), x, y);
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.into_iter().enumerate() {
        let _ = array.set_index(scope, index as u32, value.into());
    }
    result.set(array.into());
}
