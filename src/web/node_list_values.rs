pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "values", 0, values)
}
fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(items) = super::node_list::list_or_throw(scope, arguments.this()) else {
        return;
    };
    let array = v8::Array::new(scope, items.len() as i32);
    for (index, item) in items.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, (*item).into());
    }
    super::node_list::return_iterator(scope, array, result);
}
