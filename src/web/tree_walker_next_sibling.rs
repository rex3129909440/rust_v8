pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "nextSibling", 0, next_sibling)
}
fn next_sibling(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::tree_walker::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let current = v8::Local::new(scope, &record.current);
    let candidate = match super::tree_walker::visible_parent(scope, &record, current) {
        Ok(Some(parent)) => {
            let parent = v8::Local::new(scope, &parent);
            let Ok(values) = super::tree_walker::visible_children(scope, &record, parent) else {
                return;
            };
            let index = values
                .iter()
                .position(|value| v8::Local::new(scope, value).strict_equals(current.into()));
            index.and_then(|index| {
                values
                    .get(index + 1)
                    .map(|value| v8::Local::new(scope, value))
            })
        }
        Ok(None) => None,
        Err(()) => return,
    };
    super::tree_walker::return_candidate(scope, arguments.this(), candidate, result);
}
