use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlTableCellElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, CellRecord>,
}

#[derive(Clone)]
pub(crate) struct CellRecord {
    pub(crate) col_span: u32,
    pub(crate) row_span: u32,
    pub(crate) no_wrap: bool,
    pub(crate) strings: HashMap<String, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTableCellElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTableCellElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTableCellElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTableCellElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_table_cell_element_col_span_property::define(scope, prototype)?;
    super::html_table_cell_element_row_span_property::define(scope, prototype)?;
    super::html_table_cell_element_headers_property::define(scope, prototype)?;
    super::html_table_cell_element_cell_index_property::define(scope, prototype)?;
    super::html_table_cell_element_align_property::define(scope, prototype)?;
    super::html_table_cell_element_axis_property::define(scope, prototype)?;
    super::html_table_cell_element_height_property::define(scope, prototype)?;
    super::html_table_cell_element_width_property::define(scope, prototype)?;
    super::html_table_cell_element_ch_property::define(scope, prototype)?;
    super::html_table_cell_element_ch_off_property::define(scope, prototype)?;
    super::html_table_cell_element_no_wrap_property::define(scope, prototype)?;
    super::html_table_cell_element_v_align_property::define(scope, prototype)?;
    super::html_table_cell_element_bg_color_property::define(scope, prototype)?;
    super::html_table_cell_element_abbr_property::define(scope, prototype)?;
    super::html_table_cell_element_scope_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTableCellElementStore>()
        .ok_or_else(|| "HTMLTableCellElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLTableCellElement".to_owned());
    }
    super::html_element::attach(scope, object, tag_name);
    scope
        .get_slot_mut::<HtmlTableCellElementStore>()
        .ok_or_else(|| "HTMLTableCellElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CellRecord {
                col_span: 1,
                row_span: 1,
                no_wrap: false,
                strings: HashMap::new(),
            },
        );
    Ok(object)
}

pub(crate) fn is_cell(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
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
) -> Option<CellRecord> {
    scope
        .get_slot::<HtmlTableCellElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_col_span(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let value = reflected_span(scope, arguments.this(), "colspan", 1, 1000, false);
        result.set(v8::Integer::new_from_unsigned(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_col_span(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let raw = reflected_unsigned_value(scope, arguments.get(0), false);
    let value = raw.clamp(1, 1000);
    let _ = super::element::set_attribute_value(
        scope,
        arguments.this(),
        "colspan".to_owned(),
        raw.to_string(),
    );
    if let Some(record) = scope
        .get_slot_mut::<HtmlTableCellElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.col_span = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_row_span(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let value = reflected_span(scope, arguments.this(), "rowspan", 1, 65534, true);
        result.set(v8::Integer::new_from_unsigned(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_row_span(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let raw = reflected_unsigned_value(scope, arguments.get(0), true);
    let value = if raw == 0 { 0 } else { raw.min(65534) };
    let _ = super::element::set_attribute_value(
        scope,
        arguments.this(),
        "rowspan".to_owned(),
        raw.to_string(),
    );
    if let Some(record) = scope
        .get_slot_mut::<HtmlTableCellElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.row_span = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_no_wrap(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let value =
            super::element::reflected_boolean(scope, arguments.this(), "nowrap").unwrap_or(false);
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_no_wrap(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).boolean_value(scope);
    let _ = super::element::set_reflected_boolean(scope, arguments.this(), "nowrap", value);
}
pub(crate) fn get_cell_index(
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

fn reflected_unsigned_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    allow_zero: bool,
) -> u32 {
    let number = value.number_value(scope).unwrap_or(0.0);
    if number == 0.0 {
        return 0;
    }
    if !number.is_finite() {
        return if number.is_sign_positive() && !number.is_nan() {
            u32::MAX
        } else {
            1
        };
    }
    if number < 0.0 {
        return 1;
    }
    number
        .round_ties_even()
        .clamp(if allow_zero { 0.0 } else { 1.0 }, f64::from(u32::MAX)) as u32
}

fn reflected_span(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    attribute: &str,
    fallback: u32,
    maximum: u32,
    allow_zero: bool,
) -> u32 {
    let Some(raw) = super::element::attribute_value(scope, object, attribute) else {
        return fallback;
    };
    let Ok(value) = raw.trim().parse::<u64>() else {
        return fallback;
    };
    if value == 0 {
        return if allow_zero { 0 } else { fallback };
    }
    value.min(u64::from(maximum)) as u32
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
