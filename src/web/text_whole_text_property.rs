pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "wholeText", get_whole_text)
}

fn get_whole_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(data) = super::text::data_if_text(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut output = data;
    if let Some(parent) = super::node::parent(scope, arguments.this()) {
        let siblings = super::node::children(scope, parent);
        let Some(index) = siblings
            .iter()
            .position(|node| node.strict_equals(arguments.this().into()))
        else {
            return;
        };
        let mut preceding = Vec::new();
        for node in siblings[..index].iter().rev() {
            match super::text::data_if_text(scope, *node) {
                Some(data) => preceding.push(data),
                None => break,
            }
        }
        preceding.reverse();
        output = preceding.concat() + &output;
        for node in &siblings[index + 1..] {
            match super::text::data_if_text(scope, *node) {
                Some(data) => output.push_str(&data),
                None => break,
            }
        }
    }
    if let Some(value) = v8::String::new(scope, &output) {
        result.set(value.into());
    }
}
