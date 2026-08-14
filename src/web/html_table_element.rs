use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlTableElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TableRecord>,
}

#[derive(Clone)]
pub(crate) struct TableRecord {
    pub(crate) caption: Option<v8::Global<v8::Object>>,
    pub(crate) t_head: Option<v8::Global<v8::Object>>,
    pub(crate) t_foot: Option<v8::Global<v8::Object>>,
    pub(crate) t_bodies: v8::Global<v8::Object>,
    pub(crate) rows: v8::Global<v8::Object>,
    pub(crate) strings: HashMap<String, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTableElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTableElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTableElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTableElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_table_element_caption_property::define(scope, prototype)?;
    super::html_table_element_t_head_property::define(scope, prototype)?;
    super::html_table_element_t_foot_property::define(scope, prototype)?;
    super::html_table_element_t_bodies_property::define(scope, prototype)?;
    super::html_table_element_rows_property::define(scope, prototype)?;
    super::html_table_element_align_property::define(scope, prototype)?;
    super::html_table_element_border_property::define(scope, prototype)?;
    super::html_table_element_frame_property::define(scope, prototype)?;
    super::html_table_element_rules_property::define(scope, prototype)?;
    super::html_table_element_summary_property::define(scope, prototype)?;
    super::html_table_element_width_property::define(scope, prototype)?;
    super::html_table_element_bg_color_property::define(scope, prototype)?;
    super::html_table_element_cell_padding_property::define(scope, prototype)?;
    super::html_table_element_cell_spacing_property::define(scope, prototype)?;
    super::html_table_element_create_caption::define(scope, prototype)?;
    super::html_table_element_create_t_body::define(scope, prototype)?;
    super::html_table_element_create_t_foot::define(scope, prototype)?;
    super::html_table_element_create_t_head::define(scope, prototype)?;
    super::html_table_element_delete_caption::define(scope, prototype)?;
    super::html_table_element_delete_row::define(scope, prototype)?;
    super::html_table_element_delete_t_foot::define(scope, prototype)?;
    super::html_table_element_delete_t_head::define(scope, prototype)?;
    super::html_table_element_insert_row::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTableElementStore>()
        .ok_or_else(|| "HTMLTableElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLTableElement".to_owned());
    }
    super::html_element::attach(scope, object, "TABLE");
    let t_bodies = super::html_collection::create(scope, Vec::new())?;
    let rows = super::html_collection::create(scope, Vec::new())?;
    let t_bodies = v8::Global::new(scope, t_bodies);
    let rows = v8::Global::new(scope, rows);
    scope
        .get_slot_mut::<HtmlTableElementStore>()
        .ok_or_else(|| "HTMLTableElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TableRecord {
                caption: None,
                t_head: None,
                t_foot: None,
                t_bodies,
                rows,
                strings: HashMap::new(),
            },
        );
    Ok(object)
}

pub(crate) fn row_index(
    scope: &v8::PinScope<'_, '_>,
    row: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let parent = super::node::parent(scope, row)?;
    let table = if is_table(scope, parent) {
        parent
    } else {
        let grandparent = super::node::parent(scope, parent)?;
        is_table(scope, grandparent).then_some(grandparent)?
    };
    table_rows(scope, table)
        .iter()
        .position(|candidate| candidate.strict_equals(row.into()))
        .map(|index| index as i32)
}

pub(crate) fn is_table(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
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
) -> Option<TableRecord> {
    scope
        .get_slot::<HtmlTableElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn tag_name(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    super::element::record(scope, object).map(|record| record.tag_name)
}

pub(crate) fn table_rows<'s>(
    scope: &v8::PinScope<'s, '_>,
    table: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let children = super::node::children(scope, table);
    let mut rows = Vec::new();
    // HTMLTableElement.rows is not plain tree order: THEAD rows precede all
    // table/TBODY rows and TFOOT rows follow them, irrespective of where the
    // section nodes currently occur in the table child list.
    for child in &children {
        if tag_name(scope, *child).as_deref() == Some("THEAD") {
            rows.extend(super::html_table_section_element::direct_rows(
                scope, *child,
            ));
        }
    }
    for child in &children {
        if super::html_table_row_element::is_row(scope, *child) {
            rows.push(*child);
        } else if tag_name(scope, *child).as_deref() == Some("TBODY") {
            rows.extend(super::html_table_section_element::direct_rows(
                scope, *child,
            ));
        }
    }
    for child in &children {
        if tag_name(scope, *child).as_deref() == Some("TFOOT") {
            rows.extend(super::html_table_section_element::direct_rows(
                scope, *child,
            ));
        }
    }
    rows
}

pub(crate) fn table_bodies<'s>(
    scope: &v8::PinScope<'s, '_>,
    table: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::node::children(scope, table)
        .into_iter()
        .filter(|child| tag_name(scope, *child).is_some_and(|tag| tag == "TBODY"))
        .collect()
}

pub(crate) fn refresh_collections(
    scope: &mut v8::PinScope<'_, '_>,
    table: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = record(scope, table) {
        let body_collection = v8::Local::new(scope, &record.t_bodies);
        let bodies = table_bodies(scope, table);
        let _ = super::html_collection::replace(scope, body_collection, bodies);
        let row_collection = v8::Local::new(scope, &record.rows);
        let rows = table_rows(scope, table);
        let _ = super::html_collection::replace(scope, row_collection, rows);
    }
}

pub(crate) fn return_optional(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TableRecord) -> &Option<v8::Global<v8::Object>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
pub(crate) fn get_caption(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_special(s, a, r, SpecialChild::Caption);
}
pub(crate) fn get_t_head(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_special(s, a, r, SpecialChild::Head);
}
pub(crate) fn get_t_foot(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_special(s, a, r, SpecialChild::Foot);
}

fn return_special(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    kind: SpecialChild,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = special_child(scope, arguments.this(), kind) {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_caption(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_special_child(
        scope,
        arguments.this(),
        arguments.get(0),
        SpecialChild::Caption,
    );
}
pub(crate) fn set_t_head(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_special_child(
        scope,
        arguments.this(),
        arguments.get(0),
        SpecialChild::Head,
    );
}
pub(crate) fn set_t_foot(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_special_child(
        scope,
        arguments.this(),
        arguments.get(0),
        SpecialChild::Foot,
    );
}

#[derive(Clone, Copy)]
pub(crate) enum SpecialChild {
    Caption,
    Head,
    Foot,
}

pub(crate) fn set_special_child(
    scope: &mut v8::PinScope<'_, '_>,
    table: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    kind: SpecialChild,
) {
    let current = special_child(scope, table, kind).map(|value| v8::Global::new(scope, value));
    if value.is_null() {
        if let Some(current) = current {
            let _ = super::node::detach(scope, v8::Local::new(scope, current));
        }
        update_special(scope, table, kind, None);
        return;
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "The value has an invalid element type");
        return;
    };
    let valid = match kind {
        SpecialChild::Caption => super::html_table_caption_element::is_caption(scope, object),
        SpecialChild::Head => tag_name(scope, object).is_some_and(|tag| tag == "THEAD"),
        SpecialChild::Foot => tag_name(scope, object).is_some_and(|tag| tag == "TFOOT"),
    };
    if !valid {
        let (property, interface) = match kind {
            SpecialChild::Caption => ("caption", "HTMLTableCaptionElement"),
            SpecialChild::Head => ("tHead", "HTMLTableSectionElement"),
            SpecialChild::Foot => ("tFoot", "HTMLTableSectionElement"),
        };
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to set the '{property}' property on 'HTMLTableElement': Failed to convert value to '{interface}'."
            ),
        );
        return;
    }
    if let Some(current) = current {
        let _ = super::node::detach(scope, v8::Local::new(scope, current));
    }
    let insertion = match kind {
        SpecialChild::Caption => 0,
        SpecialChild::Head => 1.min(super::node::children(scope, table).len()),
        SpecialChild::Foot => super::node::children(scope, table).len(),
    };
    let global = v8::Global::new(scope, object);
    let _ = super::node::insert_child(scope, table, object, insertion);
    update_special(scope, table, kind, Some(global));
    refresh_collections(scope, table);
}

pub(crate) fn update_special(
    scope: &mut v8::PinScope<'_, '_>,
    table: v8::Local<'_, v8::Object>,
    kind: SpecialChild,
    value: Option<v8::Global<v8::Object>>,
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlTableElementStore>()
        .and_then(|store| store.records.get_mut(&table.get_identity_hash().get()))
    {
        match kind {
            SpecialChild::Caption => record.caption = value,
            SpecialChild::Head => record.t_head = value,
            SpecialChild::Foot => record.t_foot = value,
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_t_bodies(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    refresh_collections(scope, arguments.this());
    result.set(v8::Local::new(scope, &record.t_bodies).into());
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
    refresh_collections(scope, arguments.this());
    result.set(v8::Local::new(scope, &record.rows).into());
}

pub(crate) fn create_caption(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(caption) = special_child(scope, arguments.this(), SpecialChild::Caption) {
        result.set(caption.into());
        return;
    }
    match super::html_table_caption_element::create(scope) {
        Ok(caption) => {
            let global = v8::Global::new(scope, caption);
            let _ = super::node::insert_child(scope, arguments.this(), caption, 0);
            update_special(scope, arguments.this(), SpecialChild::Caption, Some(global));
            result.set(caption.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn create_section(
    scope: &mut v8::PinScope<'_, '_>,
    table: &v8::Global<v8::Object>,
    tag: &str,
    kind: Option<SpecialChild>,
) -> Option<v8::Global<v8::Object>> {
    let table = v8::Local::new(scope, table);
    let section = match super::html_table_section_element::create(scope, tag) {
        Ok(section) => section,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    let children = super::node::children(scope, table);
    let index = match tag {
        "THEAD" => children
            .iter()
            .position(|child| {
                matches!(
                    tag_name(scope, *child).as_deref(),
                    Some("THEAD" | "TBODY" | "TFOOT" | "TR")
                )
            })
            .unwrap_or(children.len()),
        "TBODY" => children
            .iter()
            .position(|child| tag_name(scope, *child).as_deref() == Some("TFOOT"))
            .unwrap_or(children.len()),
        _ => children.len(),
    };
    let section_global = v8::Global::new(scope, section);
    let global = kind.map(|_| section_global.clone());
    let _ = super::node::insert_child(scope, table, section, index);
    if let (Some(kind), Some(global)) = (kind, global) {
        update_special(scope, table, kind, Some(global));
    }
    refresh_collections(scope, table);
    Some(section_global)
}

pub(crate) fn create_t_body(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let table = v8::Global::new(scope, arguments.this());
    if let Some(section) = create_section(scope, &table, "TBODY", None) {
        result.set(v8::Local::new(scope, section).into());
    }
}
pub(crate) fn create_t_head(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let table = v8::Global::new(scope, arguments.this());
    if let Some(section) = special_child(scope, arguments.this(), SpecialChild::Head) {
        result.set(section.into());
    } else if let Some(section) = create_section(scope, &table, "THEAD", Some(SpecialChild::Head)) {
        result.set(v8::Local::new(scope, section).into());
    }
}
pub(crate) fn create_t_foot(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let table = v8::Global::new(scope, arguments.this());
    if let Some(section) = special_child(scope, arguments.this(), SpecialChild::Foot) {
        result.set(section.into());
    } else if let Some(section) = create_section(scope, &table, "TFOOT", Some(SpecialChild::Foot)) {
        result.set(v8::Local::new(scope, section).into());
    }
}

pub(crate) fn delete_caption(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    delete_special(scope, arguments.this(), SpecialChild::Caption);
}
pub(crate) fn delete_t_head(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    delete_special(scope, arguments.this(), SpecialChild::Head);
}
pub(crate) fn delete_t_foot(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    delete_special(scope, arguments.this(), SpecialChild::Foot);
}
pub(crate) fn delete_special(
    scope: &mut v8::PinScope<'_, '_>,
    table: v8::Local<'_, v8::Object>,
    kind: SpecialChild,
) {
    if let Some(current) = special_child(scope, table, kind) {
        let _ = super::node::detach(scope, current);
    }
    update_special(scope, table, kind, None);
    refresh_collections(scope, table);
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
    let rows = table_rows(scope, arguments.this());
    let requested = if arguments.get(0).is_undefined() {
        -1
    } else {
        arguments.get(0).int32_value(scope).unwrap_or(-1)
    };
    if requested < -1 || (requested != -1 && requested as usize > rows.len()) {
        let message = if requested < -1 {
            format!(
                "Failed to execute 'insertRow' on 'HTMLTableElement': The index provided ({requested}) is less than -1."
            )
        } else {
            format!(
                "Failed to execute 'insertRow' on 'HTMLTableElement': The index provided ({requested}) is greater than the number of rows in the table ({}).",
                rows.len()
            )
        };
        throw_index_size_message(scope, &message);
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

pub(crate) fn delete_row(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let rows = table_rows(scope, arguments.this());
    let requested = arguments.get(0).int32_value(scope).unwrap_or(-1);
    let index = if requested == -1 {
        rows.len().checked_sub(1)
    } else if requested >= 0 {
        Some(requested as usize)
    } else {
        None
    };
    if requested == -1 && rows.is_empty() {
        return;
    }
    let Some(index) = index.filter(|index| *index < rows.len()) else {
        throw_index_size_message(
            scope,
            &format!(
                "Failed to execute 'deleteRow' on 'HTMLTableElement': The index provided ({requested}) is greater than or equal to the number of rows in the table ({}).",
                rows.len()
            ),
        );
        return;
    };
    let _ = super::node::detach(scope, rows[index]);
    refresh_collections(scope, arguments.this());
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

pub(crate) fn special_child<'s>(
    scope: &v8::PinScope<'s, '_>,
    table: v8::Local<'s, v8::Object>,
    kind: SpecialChild,
) -> Option<v8::Local<'s, v8::Object>> {
    let expected = match kind {
        SpecialChild::Caption => "CAPTION",
        SpecialChild::Head => "THEAD",
        SpecialChild::Foot => "TFOOT",
    };
    super::node::children(scope, table)
        .into_iter()
        .find(|child| tag_name(scope, *child).as_deref() == Some(expected))
}
