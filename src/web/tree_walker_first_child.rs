pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "firstChild", 0, first_child)
}
fn first_child(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::tree_walker::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let current = v8::Local::new(scope, &record.current);
    let Ok(children) = super::tree_walker::visible_children(scope, &record, current) else {
        return;
    };
    let candidate = children.first().map(|value| v8::Local::new(scope, value));
    super::tree_walker::return_candidate(scope, arguments.this(), candidate, result);
}
