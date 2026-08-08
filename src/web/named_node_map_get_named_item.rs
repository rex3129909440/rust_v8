pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getNamedItem", 1, get_named_item)
}
fn get_named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    super::named_node_map::return_match(scope, arguments.this(), result, |record| {
        record.name.eq_ignore_ascii_case(&name)
    });
}
