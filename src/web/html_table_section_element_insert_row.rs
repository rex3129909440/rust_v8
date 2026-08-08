use super::html_table_section_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "insertRow", 0, insert_row)
}

fn insert_row(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let rows = direct_rows(scope, arguments.this());
    let requested = if arguments.get(0).is_undefined() {
        -1
    } else {
        arguments.get(0).int32_value(scope).unwrap_or(-1)
    };
    if requested < -1 || (requested != -1 && requested as usize > rows.len()) {
        throw_index_size(scope);
        return;
    }
    let index = if requested == -1 {
        rows.len()
    } else {
        requested as usize
    };
    match super::html_table_row_element::create(scope) {
        Ok(row) => {
            if super::node::insert_child(scope, arguments.this(), row, index) {
                refresh_rows(scope, arguments.this());
                result.set(row.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
