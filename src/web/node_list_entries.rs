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
    let Some(items) = super::node_list::list_or_throw(scope, arguments.this()) else {
        return;
    };
    let array = v8::Array::new(scope, items.len() as i32);
    for (index, item) in items.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        let _ = pair.set_index(
            scope,
            0,
            v8::Integer::new_from_unsigned(scope, index as u32).into(),
        );
        let _ = pair.set_index(scope, 1, (*item).into());
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    super::node_list::return_iterator(scope, array, result);
}
