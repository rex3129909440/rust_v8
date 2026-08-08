use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlTableSectionElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SectionRecord>,
}

#[derive(Clone)]
pub(crate) struct SectionRecord {
    pub(crate) rows: v8::Global<v8::Object>,
    pub(crate) strings: HashMap<String, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTableSectionElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTableSectionElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTableSectionElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTableSectionElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_table_section_element_rows_property::define(scope, prototype)?;
    super::html_table_section_element_align_property::define(scope, prototype)?;
    super::html_table_section_element_ch_property::define(scope, prototype)?;
    super::html_table_section_element_ch_off_property::define(scope, prototype)?;
    super::html_table_section_element_v_align_property::define(scope, prototype)?;
    super::html_table_section_element_delete_row::define(scope, prototype)?;
    super::html_table_section_element_insert_row::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTableSectionElementStore>()
        .ok_or_else(|| "HTMLTableSectionElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tag_name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLTableSectionElement".to_owned());
    }
    super::html_element::attach(scope, object, tag_name);
    let rows = super::html_collection::create(scope, Vec::new())?;
    let rows = v8::Global::new(scope, rows);
    scope
        .get_slot_mut::<HtmlTableSectionElementStore>()
        .ok_or_else(|| "HTMLTableSectionElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            SectionRecord {
                rows,
                strings: HashMap::new(),
            },
        );
    Ok(object)
}

pub(crate) fn is_section(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn row_index(
    scope: &v8::PinScope<'_, '_>,
    row: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let parent = super::node::parent(scope, row)?;
    if !is_section(scope, parent) {
        return None;
    }
    direct_rows(scope, parent)
        .iter()
        .position(|candidate| candidate.strict_equals(row.into()))
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
) -> Option<SectionRecord> {
    scope
        .get_slot::<HtmlTableSectionElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn direct_rows<'s>(
    scope: &v8::PinScope<'s, '_>,
    section: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::node::children(scope, section)
        .into_iter()
        .filter(|child| super::html_table_row_element::is_row(scope, *child))
        .collect()
}

pub(crate) fn refresh_rows(scope: &mut v8::PinScope<'_, '_>, section: v8::Local<'_, v8::Object>) {
    if let Some(record) = record(scope, section) {
        let collection = v8::Local::new(scope, &record.rows);
        let rows = direct_rows(scope, section);
        let _ = super::html_collection::replace(scope, collection, rows);
    }
}

pub(crate) fn get_rows(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    refresh_rows(scope, arguments.this());
    result.set(v8::Local::new(scope, &record.rows).into());
}

pub(crate) fn insert_row(
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

pub(crate) fn delete_row(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let rows = direct_rows(scope, arguments.this());
    let requested = arguments.get(0).int32_value(scope).unwrap_or(-1);
    let index = if requested == -1 {
        rows.len().checked_sub(1)
    } else if requested >= 0 {
        Some(requested as usize)
    } else {
        None
    };
    let Some(index) = index.filter(|index| *index < rows.len()) else {
        throw_index_size(scope);
        return;
    };
    let _ = super::node::detach(scope, rows[index]);
    refresh_rows(scope, arguments.this());
}

pub(crate) fn throw_index_size(scope: &mut v8::PinScope<'_, '_>) {
    match super::dom_exception::create(
        scope,
        "The index is not in the allowed range.".to_owned(),
        "IndexSizeError".to_owned(),
    ) {
        Ok(error) => {
            scope.throw_exception(error.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
