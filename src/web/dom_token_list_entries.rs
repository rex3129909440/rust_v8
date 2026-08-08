pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)
}
fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(values) = super::dom_token_list::list(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        let number = v8::Integer::new_from_unsigned(scope, index as u32);
        let _ = pair.set_index(scope, 0, number.into());
        if let Some(value) = v8::String::new(scope, value) {
            let _ = pair.set_index(scope, 1, value.into());
        }
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    super::dom_token_list::iterator_from_array(scope, array, "values", result)
}
