pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "replace", 2, replace)
}
fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(old) = super::dom_token_list::validate_token(scope, arguments.get(0)) else {
        return;
    };
    let Some(new) = super::dom_token_list::validate_token(scope, arguments.get(1)) else {
        return;
    };
    let mut replaced = false;
    if !super::dom_token_list::update(scope, arguments.this(), |values| {
        if let Some(index) = values.iter().position(|v| v == &old) {
            replaced = true;
            if values.contains(&new) {
                values.remove(index);
            } else {
                values[index] = new
            }
        }
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::dom_token_list::commit_binding(scope, arguments.this());
    result.set(v8::Boolean::new(scope, replaced).into())
}
