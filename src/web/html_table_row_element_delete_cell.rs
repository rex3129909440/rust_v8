use super::html_table_row_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "deleteCell", 1, delete_cell)
}

fn delete_cell(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let cells = direct_cells(scope, arguments.this());
    let requested = arguments.get(0).int32_value(scope).unwrap_or(-1);
    let index = if requested == -1 {
        cells.len().checked_sub(1)
    } else if requested >= 0 {
        Some(requested as usize)
    } else {
        None
    };
    let Some(index) = index.filter(|index| *index < cells.len()) else {
        throw_index_size(scope);
        return;
    };
    let _ = super::node::detach(scope, cells[index]);
    refresh_cells(scope, arguments.this());
}
