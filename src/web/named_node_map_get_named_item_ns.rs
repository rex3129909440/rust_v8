pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getNamedItemNS", 2, get_named_item_ns)
}
fn get_named_item_ns(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'getNamedItemNS' on 'NamedNodeMap': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let namespace = super::named_node_map::optional_namespace(scope, arguments.get(0));
    let local_name = crate::webidl::value_to_string(scope, arguments.get(1));
    super::named_node_map::return_match(scope, arguments.this(), result, |record| {
        record.namespace_uri == namespace && record.local_name == local_name
    });
}
