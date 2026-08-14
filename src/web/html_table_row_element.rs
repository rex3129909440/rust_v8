use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlTableRowElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, RowRecord>,
}

#[derive(Clone)]
pub(crate) struct RowRecord {
    pub(crate) cells: v8::Global<v8::Object>,
    pub(crate) strings: HashMap<String, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTableRowElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTableRowElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTableRowElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTableRowElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_table_row_element_row_index_property::define(scope, prototype)?;
    super::html_table_row_element_section_row_index_property::define(scope, prototype)?;
    super::html_table_row_element_cells_property::define(scope, prototype)?;
    super::html_table_row_element_align_property::define(scope, prototype)?;
    super::html_table_row_element_ch_property::define(scope, prototype)?;
    super::html_table_row_element_ch_off_property::define(scope, prototype)?;
    super::html_table_row_element_v_align_property::define(scope, prototype)?;
    super::html_table_row_element_bg_color_property::define(scope, prototype)?;
    super::html_table_row_element_delete_cell::define(scope, prototype)?;
    super::html_table_row_element_insert_cell::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTableRowElementStore>()
        .ok_or_else(|| "HTMLTableRowElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLTableRowElement".to_owned());
    }
    super::html_element::attach(scope, object, "TR");
    let cells = super::html_collection::create(scope, Vec::new())?;
    let cells = v8::Global::new(scope, cells);
    scope
        .get_slot_mut::<HtmlTableRowElementStore>()
        .ok_or_else(|| "HTMLTableRowElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            RowRecord {
                cells,
                strings: HashMap::new(),
            },
        );
    Ok(object)
}

pub(crate) fn is_row(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn cell_index(
    scope: &v8::PinScope<'_, '_>,
    cell: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let parent = super::node::parent(scope, cell)?;
    if !is_row(scope, parent) {
        return None;
    }
    direct_cells(scope, parent)
        .iter()
        .position(|candidate| candidate.strict_equals(cell.into()))
        .map(|index| index as i32)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RowRecord> {
    scope
        .get_slot::<HtmlTableRowElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn direct_cells<'s>(
    scope: &v8::PinScope<'s, '_>,
    row: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::node::children(scope, row)
        .into_iter()
        .filter(|child| super::html_table_cell_element::is_cell(scope, *child))
        .collect()
}

pub(crate) fn refresh_cells(scope: &mut v8::PinScope<'_, '_>, row: v8::Local<'_, v8::Object>) {
    if let Some(record) = record(scope, row) {
        let collection = v8::Local::new(scope, &record.cells);
        let cells = direct_cells(scope, row);
        let _ = super::html_collection::replace(scope, collection, cells);
    }
}

pub(crate) fn get_row_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let index = super::html_table_element::row_index(scope, arguments.this()).unwrap_or(-1);
    result.set(v8::Integer::new(scope, index).into());
}

pub(crate) fn get_section_row_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let index = super::html_table_section_element::row_index(scope, arguments.this()).unwrap_or(-1);
    result.set(v8::Integer::new(scope, index).into());
}

pub(crate) fn get_cells(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    refresh_cells(scope, arguments.this());
    result.set(v8::Local::new(scope, &record.cells).into());
}

pub(crate) fn insert_cell(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let cells = direct_cells(scope, arguments.this());
    let requested = if arguments.get(0).is_undefined() {
        -1
    } else {
        arguments.get(0).int32_value(scope).unwrap_or(-1)
    };
    if requested < -1 || (requested != -1 && requested as usize > cells.len()) {
        throw_index_size_message(
            scope,
            &format!(
                "Failed to execute 'insertCell' on 'HTMLTableRowElement': The value provided ({requested}) is outside the range [-1, {}].",
                cells.len()
            ),
        );
        return;
    }
    let index = if requested == -1 {
        cells.len()
    } else {
        requested as usize
    };
    match super::html_table_cell_element::create(scope, "TD") {
        Ok(cell) => {
            if super::node::insert_child(scope, arguments.this(), cell, index) {
                refresh_cells(scope, arguments.this());
                result.set(cell.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn delete_cell(
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
    if requested == -1 && cells.is_empty() {
        return;
    }
    let Some(index) = index.filter(|index| *index < cells.len()) else {
        throw_index_size_message(
            scope,
            &format!(
                "Failed to execute 'deleteCell' on 'HTMLTableRowElement': The value provided ({requested}) is outside the range [0, {}).",
                cells.len()
            ),
        );
        return;
    };
    let _ = super::node::detach(scope, cells[index]);
    refresh_cells(scope, arguments.this());
}

pub(crate) fn throw_index_size(scope: &mut v8::PinScope<'_, '_>) {
    throw_index_size_message(scope, "The index is not in the allowed range.");
}

pub(crate) fn throw_index_size_message(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), "IndexSizeError".to_owned()) {
        Ok(error) => {
            scope.throw_exception(error.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn get_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    attribute: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value =
        super::element::attribute_value(scope, arguments.this(), attribute).unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn set_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    attribute: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let _ =
        super::element::set_attribute_value(scope, arguments.this(), attribute.to_owned(), value);
}
