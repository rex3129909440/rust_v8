pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "item", 1, item)
}
fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(items) = super::node_list::list_or_throw(scope, arguments.this()) else {
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'item' on 'NodeList': 1 argument required, but only 0 present.",
        );
        return;
    }
    let index = arguments.get(0).uint32_value(scope).unwrap_or(0) as usize;
    if let Some(item) = items.get(index) {
        result.set((*item).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
