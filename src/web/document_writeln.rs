pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "writeln", 0, writeln)
}

fn writeln(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut markup = String::new();
    for index in 0..arguments.length() {
        markup.push_str(&crate::webidl::value_to_string(scope, arguments.get(index)));
    }
    markup.push('\n');
    if let Err(message) = super::document_write::append_markup(scope, arguments.this(), &markup) {
        crate::webidl::throw_type_error(scope, &message);
    }
}
