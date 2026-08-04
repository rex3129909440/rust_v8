use super::html_table_element::*;

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
    let rows = table_rows(scope, arguments.this());
    let requested = if arguments.get(0).is_undefined() {
        -1
    } else {
        arguments.get(0).int32_value(scope).unwrap_or(-1)
    };
    if requested < -1 || (requested != -1 && requested as usize > rows.len()) {
        throw_index_size(scope);
        return;
    }
    let row = match super::html_table_row_element::create(scope) {
        Ok(row) => row,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if rows.is_empty() {
        let table = v8::Global::new(scope, arguments.this());
        let Some(body) = create_section(scope, &table, "TBODY", None) else {
            return;
        };
        let body = v8::Local::new(scope, body);
        let _ = super::node::insert_child(scope, body, row, 0);
    } else if requested == -1 || requested as usize == rows.len() {
        let parent =
            super::node::parent(scope, *rows.last().expect("last row")).unwrap_or(arguments.this());
        let index = super::node::children(scope, parent).len();
        let _ = super::node::insert_child(scope, parent, row, index);
    } else {
        let target = rows[requested as usize];
        let parent = super::node::parent(scope, target).unwrap_or(arguments.this());
        let index = super::node::children(scope, parent)
            .iter()
            .position(|child| child.strict_equals(target.into()))
            .unwrap_or(0);
        let _ = super::node::insert_child(scope, parent, row, index);
    }
    refresh_collections(scope, arguments.this());
    result.set(row.into());
}
