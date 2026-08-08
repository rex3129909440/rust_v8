use super::html_table_cell_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "cellIndex", get_cell_index)
}

fn get_cell_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let index = super::html_table_row_element::cell_index(scope, arguments.this()).unwrap_or(-1);
    result.set(v8::Integer::new(scope, index).into());
}
