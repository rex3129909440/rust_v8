pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)
}
fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let Some(items) = super::node_list::list_or_throw(scope, arguments.this()) else {
        return;
    };
    let receiver = arguments.get(1);
    for (index, item) in items.iter().enumerate() {
        let index = v8::Integer::new_from_unsigned(scope, index as u32);
        let _ = callback.call(
            scope,
            receiver,
            &[(*item).into(), index.into(), arguments.this().into()],
        );
    }
}
