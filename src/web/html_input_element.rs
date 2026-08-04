use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct InputRecord {
    pub(crate) accept: String,
    pub(crate) alt: String,
    pub(crate) autocomplete: String,
    pub(crate) default_checked: bool,
    pub(crate) checked: bool,
    pub(crate) checked_dirty: bool,
    pub(crate) dir_name: String,
    pub(crate) disabled: bool,
    pub(crate) files: v8::Global<v8::Object>,
    pub(crate) form_action: String,
    pub(crate) form_enctype: String,
    pub(crate) form_method: String,
    pub(crate) form_no_validate: bool,
    pub(crate) form_target: String,
    pub(crate) height: u32,
    pub(crate) indeterminate: bool,
    pub(crate) max: String,
    pub(crate) max_length: i32,
    pub(crate) min: String,
    pub(crate) min_length: i32,
    pub(crate) multiple: bool,
    pub(crate) name: String,
    pub(crate) pattern: String,
    pub(crate) placeholder: String,
    pub(crate) read_only: bool,
    pub(crate) required: bool,
    pub(crate) size: u32,
    pub(crate) src: String,
    pub(crate) step: String,
    pub(crate) input_type: String,
    pub(crate) default_value: String,
    pub(crate) value: String,
    pub(crate) value_dirty: bool,
    pub(crate) width: u32,
    pub(crate) custom_validity: String,
    pub(crate) validity: v8::Global<v8::Object>,
    pub(crate) labels: v8::Global<v8::Object>,
    pub(crate) selection_start: u32,
    pub(crate) selection_end: u32,
    pub(crate) selection_direction: String,
    pub(crate) align: String,
    pub(crate) use_map: String,
    pub(crate) webkit_directory: bool,
    pub(crate) incremental: bool,
    pub(crate) popover_target: Option<v8::Global<v8::Object>>,
    pub(crate) popover_target_action: String,
    pub(crate) picker_open: bool,
}

#[derive(Default)]
pub(crate) struct HtmlInputElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, InputRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlInputElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLInputElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlInputElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLInputElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_input_element_accept_property::define(scope, prototype)?;
    super::html_input_element_alt_property::define(scope, prototype)?;
    super::html_input_element_autocomplete_property::define(scope, prototype)?;
    super::html_input_element_default_checked_property::define(scope, prototype)?;
    super::html_input_element_checked_property::define(scope, prototype)?;
    super::html_input_element_dir_name_property::define(scope, prototype)?;
    super::html_input_element_disabled_property::define(scope, prototype)?;
    super::html_input_element_form_property::define(scope, prototype)?;
    super::html_input_element_files_property::define(scope, prototype)?;
    super::html_input_element_form_action_property::define(scope, prototype)?;
    super::html_input_element_form_enctype_property::define(scope, prototype)?;
    super::html_input_element_form_method_property::define(scope, prototype)?;
    super::html_input_element_form_no_validate_property::define(scope, prototype)?;
    super::html_input_element_form_target_property::define(scope, prototype)?;
    super::html_input_element_height_property::define(scope, prototype)?;
    super::html_input_element_indeterminate_property::define(scope, prototype)?;
    super::html_input_element_list_property::define(scope, prototype)?;
    super::html_input_element_max_property::define(scope, prototype)?;
    super::html_input_element_max_length_property::define(scope, prototype)?;
    super::html_input_element_min_property::define(scope, prototype)?;
    super::html_input_element_min_length_property::define(scope, prototype)?;
    super::html_input_element_multiple_property::define(scope, prototype)?;
    super::html_input_element_name_property::define(scope, prototype)?;
    super::html_input_element_pattern_property::define(scope, prototype)?;
    super::html_input_element_placeholder_property::define(scope, prototype)?;
    super::html_input_element_read_only_property::define(scope, prototype)?;
    super::html_input_element_required_property::define(scope, prototype)?;
    super::html_input_element_size_property::define(scope, prototype)?;
    super::html_input_element_src_property::define(scope, prototype)?;
    super::html_input_element_step_property::define(scope, prototype)?;
    super::html_input_element_type_property::define(scope, prototype)?;
    super::html_input_element_default_value_property::define(scope, prototype)?;
    super::html_input_element_value_property::define(scope, prototype)?;
    super::html_input_element_value_as_date_property::define(scope, prototype)?;
    super::html_input_element_value_as_number_property::define(scope, prototype)?;
    super::html_input_element_width_property::define(scope, prototype)?;
    super::html_input_element_will_validate_property::define(scope, prototype)?;
    super::html_input_element_validity_property::define(scope, prototype)?;
    super::html_input_element_validation_message_property::define(scope, prototype)?;
    super::html_input_element_labels_property::define(scope, prototype)?;
    super::html_input_element_selection_start_property::define(scope, prototype)?;
    super::html_input_element_selection_end_property::define(scope, prototype)?;
    super::html_input_element_selection_direction_property::define(scope, prototype)?;
    super::html_input_element_align_property::define(scope, prototype)?;
    super::html_input_element_use_map_property::define(scope, prototype)?;
    super::html_input_element_webkitdirectory_property::define(scope, prototype)?;
    super::html_input_element_incremental_property::define(scope, prototype)?;
    super::html_input_element_popover_target_element_property::define(scope, prototype)?;
    super::html_input_element_popover_target_action_property::define(scope, prototype)?;
    super::html_input_element_check_validity::define(scope, prototype)?;
    super::html_input_element_report_validity::define(scope, prototype)?;
    super::html_input_element_select::define(scope, prototype)?;
    super::html_input_element_set_custom_validity::define(scope, prototype)?;
    super::html_input_element_set_range_text::define(scope, prototype)?;
    super::html_input_element_set_selection_range::define(scope, prototype)?;
    super::html_input_element_show_picker::define(scope, prototype)?;
    super::html_input_element_step_down::define(scope, prototype)?;
    super::html_input_element_step_up::define(scope, prototype)?;
    super::html_input_element_webkit_entries_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlInputElementStore>()
        .ok_or_else(|| "HTMLInputElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLInputElement".to_owned());
    }
    super::html_element::attach(scope, object, "INPUT");
    let files = super::file_list::create(scope, Vec::new())?;
    let validity =
        super::validity_state::create(scope, super::validity_state::ValidityRecord::default())?;
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, object);
    let files = v8::Global::new(scope, files);
    let validity = v8::Global::new(scope, validity);
    let labels = v8::Global::new(scope, labels);
    scope
        .get_slot_mut::<HtmlInputElementStore>()
        .ok_or_else(|| "HTMLInputElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            InputRecord {
                accept: String::new(),
                alt: String::new(),
                autocomplete: String::new(),
                default_checked: false,
                checked: false,
                checked_dirty: false,
                dir_name: String::new(),
                disabled: false,
                files,
                form_action: "about:blank".to_owned(),
                form_enctype: String::new(),
                form_method: String::new(),
                form_no_validate: false,
                form_target: String::new(),
                height: 0,
                indeterminate: false,
                max: String::new(),
                max_length: -1,
                min: String::new(),
                min_length: -1,
                multiple: false,
                name: String::new(),
                pattern: String::new(),
                placeholder: String::new(),
                read_only: false,
                required: false,
                size: 20,
                src: String::new(),
                step: String::new(),
                input_type: "text".to_owned(),
                default_value: String::new(),
                value: String::new(),
                value_dirty: false,
                width: 0,
                custom_validity: String::new(),
                validity,
                labels,
                selection_start: 0,
                selection_end: 0,
                selection_direction: "forward".to_owned(),
                align: String::new(),
                use_map: String::new(),
                webkit_directory: false,
                incremental: false,
                popover_target: None,
                popover_target_action: "toggle".to_owned(),
                picker_open: false,
            },
        );
    Ok(object)
}

pub(crate) fn reset_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<HtmlInputElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.value = record.default_value.clone();
        record.value_dirty = false;
        record.checked = record.default_checked;
        record.checked_dirty = false;
        let end = record.value.encode_utf16().count().min(u32::MAX as usize) as u32;
        record.selection_start = end;
        record.selection_end = end;
        record.selection_direction = "none".to_owned();
        record.indeterminate = false;
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

pub(crate) fn throw_range_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Some(message) = v8::String::new(scope, message) {
        let exception = v8::Exception::range_error(scope, message);
        scope.throw_exception(exception);
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<InputRecord> {
    scope
        .get_slot::<HtmlInputElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut InputRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlInputElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let Some(record) = scope
        .get_slot_mut::<HtmlInputElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return;
    };
    let text = value.unwrap_or_default();
    match name.to_ascii_lowercase().as_str() {
        "accept" => record.accept = text.to_owned(),
        "alt" => record.alt = text.to_owned(),
        "autocomplete" => record.autocomplete = text.to_owned(),
        "checked" => {
            record.default_checked = value.is_some();
            if !record.checked_dirty {
                record.checked = record.default_checked;
            }
        }
        "dirname" => record.dir_name = text.to_owned(),
        "disabled" => record.disabled = value.is_some(),
        "formaction" => record.form_action = text.to_owned(),
        "formenctype" => record.form_enctype = text.to_owned(),
        "formmethod" => record.form_method = text.to_owned(),
        "formnovalidate" => record.form_no_validate = value.is_some(),
        "formtarget" => record.form_target = text.to_owned(),
        "height" => record.height = text.parse().unwrap_or(0),
        "max" => record.max = text.to_owned(),
        "maxlength" => record.max_length = text.parse().unwrap_or(-1),
        "min" => record.min = text.to_owned(),
        "minlength" => record.min_length = text.parse().unwrap_or(-1),
        "multiple" => record.multiple = value.is_some(),
        "name" => record.name = text.to_owned(),
        "pattern" => record.pattern = text.to_owned(),
        "placeholder" => record.placeholder = text.to_owned(),
        "readonly" => record.read_only = value.is_some(),
        "required" => record.required = value.is_some(),
        "size" => record.size = text.parse().ok().filter(|size| *size > 0).unwrap_or(20),
        "src" => record.src = text.to_owned(),
        "step" => record.step = text.to_owned(),
        "type" => {
            record.input_type = normalized_type(text).to_owned();
            if record.input_type == "file" {
                record.value.clear();
            }
        }
        "value" => {
            record.default_value = text.to_owned();
            if !record.value_dirty {
                record.value = sanitize_value(&record.input_type, text.to_owned());
            }
        }
        "width" => record.width = text.parse().unwrap_or(0),
        "align" => record.align = text.to_owned(),
        "usemap" => record.use_map = text.to_owned(),
        "webkitdirectory" => record.webkit_directory = value.is_some(),
        "incremental" => record.incremental = value.is_some(),
        "popovertarget" => record.popover_target = None,
        _ => {}
    }
}

pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&InputRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut InputRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&InputRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut InputRecord, bool),
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    let Some(value) = super::element::reflected_string(scope, arguments.this(), name) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
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
    let Some(value) = super::element::reflected_boolean(scope, arguments.this(), name) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, value).into());
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

pub(crate) fn get_reflected_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
    default: u32,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(scope, arguments.this(), name)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default);
    result.set(v8::Integer::new_from_unsigned(scope, value).into());
}

pub(crate) fn set_reflected_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
    reject_zero: bool,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if reject_zero && value == 0 {
        throw_range_error(scope, "The value provided is zero");
        return;
    }
    super::element::set_reflected_string(scope, arguments.this(), name, value.to_string());
}

pub(crate) fn get_reflected_i32(
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
    let value = super::element::attribute_value(scope, arguments.this(), name)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default);
    result.set(v8::Integer::new(scope, value).into());
}

pub(crate) fn set_reflected_nonnegative_i32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).int32_value(scope).unwrap_or(-1);
    if value < 0 {
        throw_range_error(scope, "The value provided is negative");
        return;
    }
    super::element::set_reflected_string(scope, arguments.this(), name, value.to_string());
}

pub(crate) fn get_reflected_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
    default_to_base: bool,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::resolved_url_attribute(scope, arguments.this(), name)
        .unwrap_or_else(|| {
            if default_to_base {
                super::element::element_base_url(scope, arguments.this())
            } else {
                String::new()
            }
        });
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn get_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&InputRecord) -> u32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut InputRecord, u32),
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_i32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&InputRecord) -> i32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_nonnegative_i32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut InputRecord, i32),
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(-1);
    if value < 0 {
        throw_range_error(scope, "The value provided is negative");
        return;
    }
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_accept(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.accept);
}
pub(crate) fn set_accept(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.accept = v);
}
pub(crate) fn get_alt(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.alt);
}
pub(crate) fn set_alt(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.alt = v);
}
pub(crate) fn get_autocomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.autocomplete);
}
pub(crate) fn set_autocomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.autocomplete = v);
}
pub(crate) fn get_default_checked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_boolean(s, a.this(), "checked").unwrap_or(false);
    let mut r = r;
    r.set(v8::Boolean::new(s, value).into());
}
pub(crate) fn set_default_checked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(s);
    super::element::set_reflected_boolean(s, a.this(), "checked", value);
}
pub(crate) fn get_checked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.checked);
}
pub(crate) fn set_checked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(s);
    update(s, a.this(), |x| {
        x.checked = value;
        x.checked_dirty = true;
    });
}
pub(crate) fn get_dir_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.dir_name);
}
pub(crate) fn set_dir_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.dir_name = v);
}
pub(crate) fn get_disabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_reflected_boolean(s, a, r, "disabled");
}
pub(crate) fn set_disabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_boolean(s, a, "disabled");
}
pub(crate) fn get_form(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(form) = super::html_form_element::ancestor_form(scope, a.this()) {
        r.set(form.into());
    } else {
        r.set(v8::null(scope).into());
    }
}
pub(crate) fn get_files(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if record.input_type == "file" {
            r.set(v8::Local::new(scope, &record.files).into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_files(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.get(0).is_null() {
        if let Ok(files) = super::file_list::create(scope, Vec::new()) {
            let files = v8::Global::new(scope, files);
            update(scope, a.this(), |x| x.files = files);
        }
        return;
    }
    let Ok(files) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "files must be a FileList or null");
        return;
    };
    let is_file_list = files
        .get(scope, crate::webidl::string(scope, "item").unwrap().into())
        .is_some_and(|value| value.is_function());
    if !is_file_list {
        crate::webidl::throw_type_error(scope, "files must be a FileList or null");
        return;
    }
    let files = v8::Global::new(scope, files);
    update(scope, a.this(), |x| x.files = files);
}
pub(crate) fn get_form_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_action);
}
pub(crate) fn set_form_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.form_action = v);
}
pub(crate) fn get_form_enctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_enctype);
}
pub(crate) fn set_form_enctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.form_enctype = v);
}
pub(crate) fn get_form_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_method);
}
pub(crate) fn set_form_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.form_method = v);
}
pub(crate) fn get_form_no_validate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.form_no_validate);
}
pub(crate) fn set_form_no_validate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.form_no_validate = v);
}
pub(crate) fn get_form_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_target);
}
pub(crate) fn set_form_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.form_target = v);
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_u32(s, a, r, |x| x.height);
}
pub(crate) fn set_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_u32(s, a, |x, v| x.height = v);
}
pub(crate) fn get_indeterminate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.indeterminate);
}
pub(crate) fn set_indeterminate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.indeterminate = v);
}
pub(crate) fn get_list(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(list_id) = super::element::attribute_value(scope, a.this(), "list") else {
        r.set(v8::null(scope).into());
        return;
    };
    let candidates = if super::node::is_connected(scope, a.this()) {
        super::node::owner_document(scope, a.this())
            .map(|document| super::document::document_descendants(scope, document))
            .unwrap_or_default()
    } else {
        let mut root = a.this();
        while let Some(parent) = super::node::parent(scope, root) {
            root = parent;
        }
        let mut candidates = super::dom_selector::descendants(scope, root);
        candidates.insert(0, root);
        candidates
    };
    let data_list = candidates.into_iter().find(|candidate| {
        super::html_data_list_element::is_data_list(scope, *candidate)
            && super::element::attribute_value(scope, *candidate, "id").as_deref()
                == Some(list_id.as_str())
    });
    match data_list {
        Some(data_list) => r.set(data_list.into()),
        None => r.set(v8::null(scope).into()),
    }
}
pub(crate) fn get_max(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.max);
}
pub(crate) fn set_max(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.max = v);
}
pub(crate) fn get_max_length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_i32(s, a, r, |x| x.max_length);
}
pub(crate) fn set_max_length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_nonnegative_i32(s, a, |x, v| x.max_length = v);
}
pub(crate) fn get_min(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.min);
}
pub(crate) fn set_min(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.min = v);
}
pub(crate) fn get_min_length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_i32(s, a, r, |x| x.min_length);
}
pub(crate) fn set_min_length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_nonnegative_i32(s, a, |x, v| x.min_length = v);
}
pub(crate) fn get_multiple(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.multiple);
}
pub(crate) fn set_multiple(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.multiple = v);
}
pub(crate) fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_reflected_string(s, a, r, "name");
}
pub(crate) fn set_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(s, a, "name");
}
pub(crate) fn get_pattern(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.pattern);
}
pub(crate) fn set_pattern(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.pattern = v);
}
pub(crate) fn get_placeholder(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_reflected_string(s, a, r, "placeholder");
}
pub(crate) fn set_placeholder(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(s, a, "placeholder");
}
pub(crate) fn get_read_only(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.read_only);
}
pub(crate) fn set_read_only(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.read_only = v);
}
pub(crate) fn get_required(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_reflected_boolean(s, a, r, "required");
}
pub(crate) fn set_required(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_boolean(s, a, "required");
}
pub(crate) fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_u32(s, a, r, |x| x.size);
}
pub(crate) fn set_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).uint32_value(s).unwrap_or(20);
    if value == 0 {
        throw_range_error(s, "The value provided is zero");
    } else {
        update(s, a.this(), |x| x.size = value);
    }
}
pub(crate) fn get_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.src);
}
pub(crate) fn set_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.src = v);
}
pub(crate) fn get_step(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.step);
}
pub(crate) fn set_step(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.step = v);
}

pub(crate) fn normalized_type(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "button" => "button",
        "checkbox" => "checkbox",
        "color" => "color",
        "date" => "date",
        "datetime-local" => "datetime-local",
        "email" => "email",
        "file" => "file",
        "hidden" => "hidden",
        "image" => "image",
        "month" => "month",
        "number" => "number",
        "password" => "password",
        "radio" => "radio",
        "range" => "range",
        "reset" => "reset",
        "search" => "search",
        "submit" => "submit",
        "tel" => "tel",
        "text" => "text",
        "time" => "time",
        "url" => "url",
        "week" => "week",
        _ => "text",
    }
}
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let raw = super::element::reflected_string(s, a.this(), "type").unwrap_or_default();
    let value = normalized_type(&raw);
    if let Some(value) = v8::String::new(s, value) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let raw = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "type", raw);
}
pub(crate) fn get_default_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_reflected_string(s, a, r, "value");
}
pub(crate) fn set_default_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "value", value);
}
pub(crate) fn sanitize_value(input_type: &str, value: String) -> String {
    if input_type == "number" || input_type == "range" {
        if value.is_empty() || value.parse::<f64>().is_ok() {
            value
        } else {
            String::new()
        }
    } else if input_type == "date" {
        if parse_date(&value).is_some() {
            value
        } else {
            String::new()
        }
    } else {
        value
    }
}
pub(crate) fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.value);
}
pub(crate) fn set_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if x.input_type == "file" && !value.is_empty() {
            return;
        }
        x.value = sanitize_value(&x.input_type, value);
        x.value_dirty = true;
        let length = x.value.chars().count() as u32;
        x.selection_start = length;
        x.selection_end = length;
        x.selection_direction = "forward".to_owned();
    });
}

pub(crate) fn parse_date(value: &str) -> Option<(i32, u32, u32)> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || year < 1 || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let limit = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if day == 0 || day > limit {
        None
    } else {
        Some((year, month, day))
    }
}
pub(crate) fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let shifted_month = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe - 719468) as i64
}
pub(crate) fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i32::from(month <= 2);
    (year, month as u32, day as u32)
}
pub(crate) fn numeric_value(record: &InputRecord) -> f64 {
    if record.input_type == "date" {
        parse_date(&record.value)
            .map(|(y, m, d)| days_from_civil(y, m, d) as f64 * 86_400_000.0)
            .unwrap_or(f64::NAN)
    } else if matches!(record.input_type.as_str(), "number" | "range") {
        record.value.parse::<f64>().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}
pub(crate) fn get_value_as_date(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.input_type == "date" {
        let value = numeric_value(&record);
        if value.is_finite() {
            if let Some(date) = v8::Date::new(scope, value) {
                r.set(date.into());
                return;
            }
        }
    }
    r.set(v8::null(scope).into());
}
pub(crate) fn set_value_as_date(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.input_type != "date" {
        crate::webidl::throw_type_error(scope, "valueAsDate is not applicable to this input type");
        return;
    }
    if a.get(0).is_null() {
        update(scope, a.this(), |x| x.value.clear());
        return;
    }
    if !a.get(0).is_date() {
        crate::webidl::throw_type_error(scope, "valueAsDate must be a Date or null");
        return;
    }
    let millis = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !millis.is_finite() {
        update(scope, a.this(), |x| x.value.clear());
        return;
    }
    let (year, month, day) = civil_from_days((millis / 86_400_000.0).floor() as i64);
    let value = format!("{year:04}-{month:02}-{day:02}");
    update(scope, a.this(), |x| x.value = value);
}
pub(crate) fn get_value_as_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, numeric_value(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_value_as_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let number = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !number.is_finite() {
        update(scope, a.this(), |x| x.value.clear());
        return;
    }
    let value = if current.input_type == "date" {
        let (year, month, day) = civil_from_days((number / 86_400_000.0).floor() as i64);
        format!("{year:04}-{month:02}-{day:02}")
    } else if matches!(current.input_type.as_str(), "number" | "range") {
        format_number(number)
    } else {
        crate::webidl::throw_type_error(
            scope,
            "valueAsNumber is not applicable to this input type",
        );
        return;
    };
    update(scope, a.this(), |x| x.value = value);
}
pub(crate) fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_u32(s, a, r, |x| x.width);
}
pub(crate) fn set_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_u32(s, a, |x, v| x.width = v);
}

pub(crate) fn will_validate(record: &InputRecord) -> bool {
    !record.disabled && !matches!(record.input_type.as_str(), "hidden" | "button" | "reset")
}
pub(crate) fn validity_record(record: &InputRecord) -> super::validity_state::ValidityRecord {
    let value_missing = record.required
        && if matches!(record.input_type.as_str(), "checkbox" | "radio") {
            !record.checked
        } else {
            record.value.is_empty()
        };
    let number = numeric_value(record);
    let min = record.min.parse::<f64>().ok();
    let max = record.max.parse::<f64>().ok();
    let range_underflow = number.is_finite() && min.is_some_and(|minimum| number < minimum);
    let range_overflow = number.is_finite() && max.is_some_and(|maximum| number > maximum);
    let step = record.step.parse::<f64>().ok().filter(|step| *step > 0.0);
    let base = min.unwrap_or(0.0);
    let step_mismatch = number.is_finite()
        && step.is_some_and(|step| {
            ((number - base) / step - ((number - base) / step).round()).abs() > 1e-9
        });
    let length = record.value.chars().count() as i32;
    let too_long = record.max_length >= 0 && length > record.max_length;
    let too_short =
        record.min_length >= 0 && !record.value.is_empty() && length < record.min_length;
    let type_mismatch = !record.value.is_empty()
        && ((record.input_type == "email"
            && (!record.value.contains('@')
                || record.value.starts_with('@')
                || record.value.ends_with('@')))
            || (record.input_type == "url" && !record.value.contains("://")));
    let pattern_mismatch = !record.pattern.is_empty()
        && !record.value.is_empty()
        && record.pattern != ".*"
        && record.value != record.pattern;
    super::validity_state::ValidityRecord {
        value_missing,
        type_mismatch,
        pattern_mismatch,
        too_long,
        too_short,
        range_underflow,
        range_overflow,
        step_mismatch,
        bad_input: false,
        custom_error: !record.custom_validity.is_empty(),
    }
}
pub(crate) fn invalid(record: &InputRecord) -> bool {
    let v = validity_record(record);
    v.value_missing
        || v.type_mismatch
        || v.pattern_mismatch
        || v.too_long
        || v.too_short
        || v.range_underflow
        || v.range_overflow
        || v.step_mismatch
        || v.bad_input
        || v.custom_error
}
pub(crate) fn get_will_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, will_validate(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let validity = v8::Local::new(scope, &record.validity);
    let _ = super::validity_state::replace(scope, validity, validity_record(&record));
    r.set(validity.into());
}
pub(crate) fn validation_message(record: &InputRecord) -> String {
    if !record.custom_validity.is_empty() {
        record.custom_validity.clone()
    } else {
        let v = validity_record(record);
        if v.value_missing {
            "Please fill out this field.".to_owned()
        } else if v.range_underflow {
            format!("Value must be greater than or equal to {}.", record.min)
        } else if v.range_overflow {
            format!("Value must be less than or equal to {}.", record.max)
        } else if v.step_mismatch {
            "Please enter a valid value.".to_owned()
        } else if v.type_mismatch {
            "Please enter a valid value.".to_owned()
        } else if v.pattern_mismatch {
            "Please match the requested format.".to_owned()
        } else {
            String::new()
        }
    }
}
pub(crate) fn get_validation_message(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        let message = if will_validate(&record) {
            validation_message(&record)
        } else {
            String::new()
        };
        if let Some(value) = v8::String::new(scope, &message) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_labels(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &record.labels).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn supports_selection(input_type: &str) -> bool {
    matches!(input_type, "text" | "search" | "tel" | "url" | "password")
}
pub(crate) fn get_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if supports_selection(&record.input_type) {
            r.set(v8::Integer::new_from_unsigned(scope, record.selection_start).into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).uint32_value(scope).unwrap_or(0);
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    update(scope, a.this(), |x| {
        let limit = x.value.chars().count() as u32;
        x.selection_start = value.min(limit);
        if x.selection_end < x.selection_start {
            x.selection_end = x.selection_start;
        }
    });
}
pub(crate) fn get_selection_end(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if supports_selection(&record.input_type) {
            r.set(v8::Integer::new_from_unsigned(scope, record.selection_end).into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_selection_end(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).uint32_value(scope).unwrap_or(0);
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    update(scope, a.this(), |x| {
        let limit = x.value.chars().count() as u32;
        x.selection_end = value.min(limit);
        if x.selection_start > x.selection_end {
            x.selection_start = x.selection_end;
        }
    });
}
pub(crate) fn get_selection_direction(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if supports_selection(&record.input_type) {
            if let Some(value) = v8::String::new(scope, &record.selection_direction) {
                r.set(value.into());
            }
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_selection_direction(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    let value = if matches!(value.as_str(), "forward" | "backward" | "none") {
        value
    } else {
        "none".to_owned()
    };
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    update(scope, a.this(), |x| x.selection_direction = value);
}
pub(crate) fn get_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.align);
}
pub(crate) fn set_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.align = v);
}
pub(crate) fn get_use_map(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.use_map);
}
pub(crate) fn set_use_map(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.use_map = v);
}
pub(crate) fn get_webkit_directory(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.webkit_directory);
}
pub(crate) fn set_webkit_directory(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.webkit_directory = v);
}
pub(crate) fn get_incremental(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.incremental);
}
pub(crate) fn set_incremental(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.incremental = v);
}
pub(crate) fn get_popover_target_element(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(target) = record.popover_target {
            r.set(v8::Local::new(scope, &target).into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_popover_target_element(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = if a.get(0).is_null() {
        None
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
            crate::webidl::throw_type_error(
                scope,
                "popoverTargetElement must be an Element or null",
            );
            return;
        };
        if super::element::record(scope, object).is_none() {
            crate::webidl::throw_type_error(
                scope,
                "popoverTargetElement must be an Element or null",
            );
            return;
        }
        Some(v8::Global::new(scope, object))
    };
    update(scope, a.this(), |x| x.popover_target = target);
}
pub(crate) fn get_popover_target_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.popover_target_action);
}
pub(crate) fn set_popover_target_action(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    let value = if matches!(value.as_str(), "show" | "hide" | "toggle") {
        value
    } else {
        "toggle".to_owned()
    };
    update(scope, a.this(), |x| x.popover_target_action = value);
}

pub(crate) fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, !will_validate(&record) || !invalid(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn report_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    check_validity(scope, a, r);
}
pub(crate) fn select(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if supports_selection(&current.input_type) {
        update(scope, a.this(), |x| {
            x.selection_start = 0;
            x.selection_end = x.value.chars().count() as u32;
            x.selection_direction = "forward".to_owned();
        });
    }
}
pub(crate) fn set_custom_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    update(scope, a.this(), |x| x.custom_validity = value);
}
pub(crate) fn set_range_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let replacement = crate::webidl::value_to_string(scope, a.get(0));
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    let length = current.value.chars().count() as u32;
    let (start, end) = if a.length() >= 3 {
        (
            a.get(1).uint32_value(scope).unwrap_or(0).min(length),
            a.get(2).uint32_value(scope).unwrap_or(0).min(length),
        )
    } else {
        (current.selection_start, current.selection_end)
    };
    if start > end {
        throw_range_error(scope, "The start index is greater than the end index");
        return;
    }
    let selection_mode = if a.length() >= 4 {
        crate::webidl::value_to_string(scope, a.get(3))
    } else {
        "preserve".to_owned()
    };
    let before = current
        .value
        .chars()
        .take(start as usize)
        .collect::<String>();
    let after = current.value.chars().skip(end as usize).collect::<String>();
    let replacement_length = replacement.chars().count() as u32;
    let new_value = format!("{before}{replacement}{after}");
    let replaced = end - start;
    let (selection_start, selection_end) = match selection_mode.as_str() {
        "select" => (start, start + replacement_length),
        "start" => (start, start),
        "end" => (start + replacement_length, start + replacement_length),
        _ => {
            let adjust = replacement_length as i64 - replaced as i64;
            let adjust_index = |index: u32| {
                if index <= start {
                    index
                } else if index >= end {
                    (index as i64 + adjust).max(0) as u32
                } else {
                    start + replacement_length
                }
            };
            (
                adjust_index(current.selection_start),
                adjust_index(current.selection_end),
            )
        }
    };
    update(scope, a.this(), |x| {
        x.value = new_value;
        x.value_dirty = true;
        x.selection_start = selection_start;
        x.selection_end = selection_end;
        x.selection_direction = "forward".to_owned();
    });
}
pub(crate) fn set_selection_range(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !supports_selection(&current.input_type) {
        crate::webidl::throw_type_error(scope, "The input type does not support selection");
        return;
    }
    let length = current.value.chars().count() as u32;
    let start = a.get(0).uint32_value(scope).unwrap_or(0).min(length);
    let end = a.get(1).uint32_value(scope).unwrap_or(0).min(length);
    let start = start.min(end);
    let direction = if a.length() > 2 {
        let value = crate::webidl::value_to_string(scope, a.get(2));
        if matches!(value.as_str(), "forward" | "backward" | "none") {
            value
        } else {
            "none".to_owned()
        }
    } else {
        "none".to_owned()
    };
    update(scope, a.this(), |x| {
        x.selection_start = start;
        x.selection_end = end;
        x.selection_direction = direction;
    });
}
pub(crate) fn show_picker(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |x| x.picker_open = true);
}
pub(crate) fn apply_step(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    direction: f64,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !matches!(current.input_type.as_str(), "number" | "range" | "date") {
        crate::webidl::throw_type_error(scope, "The input type does not support stepping");
        return;
    }
    let count = if a.length() == 0 {
        1.0
    } else {
        a.get(0).number_value(scope).unwrap_or(1.0)
    };
    let default_step = if current.input_type == "date" {
        86_400_000.0
    } else {
        1.0
    };
    let step = if current.step == "any" {
        crate::webidl::throw_type_error(scope, "The input has step='any'");
        return;
    } else {
        current
            .step
            .parse::<f64>()
            .ok()
            .filter(|v| *v > 0.0)
            .map(|v| {
                if current.input_type == "date" {
                    v * 86_400_000.0
                } else {
                    v
                }
            })
            .unwrap_or(default_step)
    };
    let base = if current.input_type == "date" {
        parse_date(&current.min)
            .map(|(y, m, d)| days_from_civil(y, m, d) as f64 * 86_400_000.0)
            .unwrap_or(0.0)
    } else {
        current.min.parse::<f64>().unwrap_or(0.0)
    };
    let existing = numeric_value(&current);
    let next = if existing.is_finite() {
        existing + direction * count * step
    } else {
        base
    };
    let value = if current.input_type == "date" {
        let (year, month, day) = civil_from_days((next / 86_400_000.0).floor() as i64);
        format!("{year:04}-{month:02}-{day:02}")
    } else {
        format_number(next)
    };
    update(scope, a.this(), |x| x.value = value);
}
pub(crate) fn step_down(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    apply_step(scope, a, -1.0);
}
pub(crate) fn step_up(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    apply_step(scope, a, 1.0);
}
pub(crate) fn get_webkit_entries(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(v8::Array::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
