use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlTextAreaElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TextAreaRecord>,
}

#[derive(Clone)]
pub(crate) struct TextAreaRecord {
    pub(crate) strings: HashMap<String, String>,
    pub(crate) numbers: HashMap<String, i32>,
    pub(crate) booleans: HashMap<String, bool>,
    pub(crate) default_value: String,
    pub(crate) value: String,
    pub(crate) value_dirty: bool,
    pub(crate) selection_start: u32,
    pub(crate) selection_end: u32,
    pub(crate) selection_direction: String,
    pub(crate) custom_validity: String,
    pub(crate) validity: v8::Global<v8::Object>,
    pub(crate) labels: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTextAreaElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTextAreaElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTextAreaElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTextAreaElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_text_area_element_autocomplete_property::define(scope, prototype)?;
    super::html_text_area_element_cols_property::define(scope, prototype)?;
    super::html_text_area_element_dir_name_property::define(scope, prototype)?;
    super::html_text_area_element_disabled_property::define(scope, prototype)?;
    super::html_text_area_element_form_property::define(scope, prototype)?;
    super::html_text_area_element_max_length_property::define(scope, prototype)?;
    super::html_text_area_element_min_length_property::define(scope, prototype)?;
    super::html_text_area_element_name_property::define(scope, prototype)?;
    super::html_text_area_element_placeholder_property::define(scope, prototype)?;
    super::html_text_area_element_read_only_property::define(scope, prototype)?;
    super::html_text_area_element_required_property::define(scope, prototype)?;
    super::html_text_area_element_rows_property::define(scope, prototype)?;
    super::html_text_area_element_wrap_property::define(scope, prototype)?;
    super::html_text_area_element_type_property::define(scope, prototype)?;
    super::html_text_area_element_default_value_property::define(scope, prototype)?;
    super::html_text_area_element_value_property::define(scope, prototype)?;
    super::html_text_area_element_text_length_property::define(scope, prototype)?;
    super::html_text_area_element_will_validate_property::define(scope, prototype)?;
    super::html_text_area_element_validity_property::define(scope, prototype)?;
    super::html_text_area_element_validation_message_property::define(scope, prototype)?;
    super::html_text_area_element_labels_property::define(scope, prototype)?;
    super::html_text_area_element_selection_start_property::define(scope, prototype)?;
    super::html_text_area_element_selection_end_property::define(scope, prototype)?;
    super::html_text_area_element_selection_direction_property::define(scope, prototype)?;
    super::html_text_area_element_check_validity::define(scope, prototype)?;
    super::html_text_area_element_report_validity::define(scope, prototype)?;
    super::html_text_area_element_select::define(scope, prototype)?;
    super::html_text_area_element_set_custom_validity::define(scope, prototype)?;
    super::html_text_area_element_set_range_text::define(scope, prototype)?;
    super::html_text_area_element_set_selection_range::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .ok_or_else(|| "HTMLTextAreaElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLTextAreaElement".to_owned());
    }
    super::html_element::attach(scope, object, "TEXTAREA");
    let validity =
        super::validity_state::create(scope, super::validity_state::ValidityRecord::default())?;
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, object);
    let validity = v8::Global::new(scope, validity);
    let labels = v8::Global::new(scope, labels);
    let mut strings = HashMap::new();
    strings.insert("autocomplete".to_owned(), String::new());
    strings.insert("dirName".to_owned(), String::new());
    strings.insert("name".to_owned(), String::new());
    strings.insert("placeholder".to_owned(), String::new());
    strings.insert("wrap".to_owned(), String::new());
    let mut numbers = HashMap::new();
    numbers.insert("cols".to_owned(), 20);
    numbers.insert("maxLength".to_owned(), -1);
    numbers.insert("minLength".to_owned(), -1);
    numbers.insert("rows".to_owned(), 2);
    let mut booleans = HashMap::new();
    booleans.insert("disabled".to_owned(), false);
    booleans.insert("readOnly".to_owned(), false);
    booleans.insert("required".to_owned(), false);
    scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .ok_or_else(|| "HTMLTextAreaElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TextAreaRecord {
                strings,
                numbers,
                booleans,
                default_value: String::new(),
                value: String::new(),
                value_dirty: false,
                selection_start: 0,
                selection_end: 0,
                selection_direction: "forward".to_owned(),
                custom_validity: String::new(),
                validity,
                labels,
            },
        );
    Ok(object)
}

pub(crate) fn reset_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    let default_value = super::node::node_text(scope, object);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.value = default_value;
        record.value_dirty = false;
        let end = record.value.encode_utf16().count().min(u32::MAX as usize) as u32;
        record.selection_start = end;
        record.selection_end = end;
        record.selection_direction = "none".to_owned();
        true
    } else {
        false
    }
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
) -> Option<TextAreaRecord> {
    scope
        .get_slot::<HtmlTextAreaElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return;
    };
    let text = value.unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "autocomplete" => {
            record
                .strings
                .insert("autocomplete".to_owned(), text.to_owned());
        }
        "dirname" => {
            record.strings.insert("dirName".to_owned(), text.to_owned());
        }
        "name" | "placeholder" | "wrap" => {
            record.strings.insert(lower, text.to_owned());
        }
        "disabled" => {
            record
                .booleans
                .insert("disabled".to_owned(), value.is_some());
        }
        "readonly" => {
            record
                .booleans
                .insert("readOnly".to_owned(), value.is_some());
        }
        "required" => {
            record
                .booleans
                .insert("required".to_owned(), value.is_some());
        }
        "cols" => {
            record.numbers.insert(
                "cols".to_owned(),
                text.parse().ok().filter(|number| *number > 0).unwrap_or(20),
            );
        }
        "rows" => {
            record.numbers.insert(
                "rows".to_owned(),
                text.parse().ok().filter(|number| *number > 0).unwrap_or(2),
            );
        }
        "maxlength" => {
            record
                .numbers
                .insert("maxLength".to_owned(), text.parse().unwrap_or(-1));
        }
        "minlength" => {
            record
                .numbers
                .insert("minLength".to_owned(), text.parse().unwrap_or(-1));
        }
        _ => {}
    }
}

pub(crate) fn get_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_string(scope, arguments.this(), name).unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn set_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::element::set_reflected_string(scope, arguments.this(), name, value);
}

pub(crate) fn get_reflected_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    if let Some(value) = record(scope, arguments.this())
        .and_then(|_| super::element::reflected_boolean(scope, arguments.this(), name))
    {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_reflected_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).boolean_value(scope);
    super::element::set_reflected_boolean(scope, arguments.this(), name, value);
}

pub(crate) fn get_reflected_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
    default: i32,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let number = super::element::attribute_value(scope, arguments.this(), name)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default);
    result.set(v8::Integer::new(scope, number).into());
}

pub(crate) fn set_reflected_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
    positive: bool,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).int32_value(scope).unwrap_or(0);
    if positive && value <= 0 {
        crate::webidl::throw_type_error(scope, "Value must be greater than zero");
        return;
    }
    super::element::set_reflected_string(scope, arguments.this(), name, value.to_string());
}

pub(crate) fn get_form(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(form) = super::html_form_element::ancestor_form(scope, arguments.this()) {
        result.set(form.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(value) = v8::String::new(scope, "textarea") {
        result.set(value.into());
    }
}

pub(crate) fn return_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextAreaRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_default_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    } else if let Some(value) = v8::String::new(s, &super::node::node_text(s, a.this())) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_default_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let was_dirty = record(scope, arguments.this()).is_some_and(|record| record.value_dirty);
    for child in super::node::children(scope, arguments.this()) {
        super::node::detach(scope, child);
    }
    if !value.is_empty()
        && let Ok(text) = super::text::create(scope, value.clone())
    {
        if let Some(document) = super::node::owner_document(scope, arguments.this()) {
            super::node::set_owner_document(scope, text, document);
        }
        let _ = super::node::insert_node(scope, arguments.this(), text, 0);
    }
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.default_value = value.clone();
        if !was_dirty {
            record.value = value;
        }
    }
}
pub(crate) fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let value = if record.value_dirty {
        record.value
    } else {
        super::node::node_text(s, a.this())
    };
    if let Some(value) = v8::String::new(s, &value) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let end = text_len(&value);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.value = value;
        record.value_dirty = true;
        record.selection_start = end;
        record.selection_end = end;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn text_len(value: &str) -> u32 {
    value.encode_utf16().count().min(u32::MAX as usize) as u32
}

pub(crate) fn get_text_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, text_len(&record.value)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn is_candidate(record: &TextAreaRecord) -> bool {
    !record.booleans.get("disabled").copied().unwrap_or(false)
        && !record.booleans.get("readOnly").copied().unwrap_or(false)
}

pub(crate) fn validity_record(record: &TextAreaRecord) -> super::validity_state::ValidityRecord {
    let length = text_len(&record.value) as i32;
    let required = record.booleans.get("required").copied().unwrap_or(false);
    let max_length = record.numbers.get("maxLength").copied().unwrap_or(-1);
    let min_length = record.numbers.get("minLength").copied().unwrap_or(-1);
    super::validity_state::ValidityRecord {
        value_missing: required && record.value.is_empty(),
        too_long: max_length >= 0 && length > max_length,
        too_short: min_length >= 0 && !record.value.is_empty() && length < min_length,
        custom_error: !record.custom_validity.is_empty(),
        ..super::validity_state::ValidityRecord::default()
    }
}

pub(crate) fn is_valid(record: &TextAreaRecord) -> bool {
    let value = validity_record(record);
    !value.value_missing && !value.too_long && !value.too_short && !value.custom_error
}

pub(crate) fn get_will_validate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, is_candidate(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let validity = v8::Local::new(scope, &record.validity);
    let _ = super::validity_state::replace(scope, validity, validity_record(&record));
    result.set(validity.into());
}

pub(crate) fn get_validation_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let message = if !record.custom_validity.is_empty() {
        record.custom_validity
    } else if record.booleans.get("required").copied().unwrap_or(false) && record.value.is_empty() {
        "Please fill out this field.".to_owned()
    } else {
        String::new()
    };
    if let Some(value) = v8::String::new(scope, &message) {
        result.set(value.into());
    }
}

pub(crate) fn get_labels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.labels).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.selection_start).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_selection_end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.selection_end).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_selection_direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |x| &x.selection_direction);
}

pub(crate) fn set_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        let value = requested.min(text_len(&record.value));
        record.selection_start = value;
        if value > record.selection_end {
            record.selection_end = value;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_selection_end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        let value = requested.min(text_len(&record.value));
        record.selection_end = value;
        if value < record.selection_start {
            record.selection_start = value;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_selection_direction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if value == "forward" || value == "backward" || value == "none" {
        value
    } else {
        "none".to_owned()
    };
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.selection_direction = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, !is_candidate(&record) || is_valid(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn report_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    check_validity(scope, arguments, result);
}
pub(crate) fn select(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.selection_start = 0;
        record.selection_end = text_len(&record.value);
        record.selection_direction = "none".to_owned();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_custom_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let message = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.custom_validity = message;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_selection_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let start = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let end = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let direction = if arguments.get(2).is_undefined() {
        "none".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(2))
    };
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        let length = text_len(&record.value);
        record.selection_end = end.min(length);
        record.selection_start = start.min(record.selection_end);
        record.selection_direction = if direction == "forward" || direction == "backward" {
            direction
        } else {
            "none".to_owned()
        };
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_range_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let replacement = crate::webidl::value_to_string(scope, arguments.get(0));
    let snapshot = record(scope, arguments.this());
    let Some(snapshot) = snapshot else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let length = text_len(&snapshot.value);
    let (start, end) = if arguments.length() >= 3 {
        (
            arguments
                .get(1)
                .uint32_value(scope)
                .unwrap_or(0)
                .min(length),
            arguments
                .get(2)
                .uint32_value(scope)
                .unwrap_or(0)
                .min(length),
        )
    } else {
        (snapshot.selection_start, snapshot.selection_end)
    };
    if start > end {
        crate::webidl::throw_type_error(scope, "The start index exceeds the end index");
        return;
    }
    let mode = if arguments.length() >= 4 {
        crate::webidl::value_to_string(scope, arguments.get(3))
    } else {
        "preserve".to_owned()
    };
    let chars = snapshot.value.chars().collect::<Vec<_>>();
    let safe_start = (start as usize).min(chars.len());
    let safe_end = (end as usize).min(chars.len());
    let before = chars[..safe_start].iter().collect::<String>();
    let after = chars[safe_end..].iter().collect::<String>();
    let value = format!("{before}{replacement}{after}");
    let inserted_end = start.saturating_add(text_len(&replacement));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.value = value;
        match mode.as_str() {
            "select" => {
                record.selection_start = start;
                record.selection_end = inserted_end;
            }
            "start" => {
                record.selection_start = start;
                record.selection_end = start;
            }
            "end" => {
                record.selection_start = inserted_end;
                record.selection_end = inserted_end;
            }
            _ => {
                let removed = end - start;
                let inserted = text_len(&replacement);
                record.selection_start =
                    adjust_position(snapshot.selection_start, start, end, inserted, removed);
                record.selection_end =
                    adjust_position(snapshot.selection_end, start, end, inserted, removed);
            }
        }
    }
}

pub(crate) fn adjust_position(
    position: u32,
    start: u32,
    end: u32,
    inserted: u32,
    removed: u32,
) -> u32 {
    if position <= start {
        position
    } else if position >= end {
        position.saturating_sub(removed).saturating_add(inserted)
    } else {
        start.saturating_add(inserted)
    }
}
