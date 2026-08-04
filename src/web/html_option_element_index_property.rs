use super::html_option_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "index", get_index)
}

fn get_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let index = super::html_select_element::containing_select(scope, arguments.this())
            .and_then(|select| {
                super::html_select_element::options_snapshot(scope, select)
                    .iter()
                    .position(|option| option.strict_equals(arguments.this().into()))
            })
            .map(|index| index as i32)
            .unwrap_or(-1);
        result.set(v8::Integer::new(scope, index).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
