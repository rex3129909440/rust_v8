use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlSelectElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SelectRecord>,
}
#[derive(Clone)]
pub(crate) struct SelectRecord {
    pub(crate) autocomplete: String,
    pub(crate) disabled: bool,
    pub(crate) multiple: bool,
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) size: u32,
    pub(crate) custom_validity: String,
    pub(crate) options: v8::Global<v8::Object>,
    pub(crate) selected_options: v8::Global<v8::Object>,
    pub(crate) validity: v8::Global<v8::Object>,
    pub(crate) labels: v8::Global<v8::Object>,
    pub(crate) picker_open: bool,
    pub(crate) selection_explicit: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlSelectElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLSelectElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlSelectElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLSelectElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_select_element_autocomplete_property::define(scope, p)?;
    super::html_select_element_disabled_property::define(scope, p)?;
    super::html_select_element_form_property::define(scope, p)?;
    super::html_select_element_multiple_property::define(scope, p)?;
    super::html_select_element_name_property::define(scope, p)?;
    super::html_select_element_required_property::define(scope, p)?;
    super::html_select_element_size_property::define(scope, p)?;
    super::html_select_element_type_property::define(scope, p)?;
    super::html_select_element_options_property::define(scope, p)?;
    super::html_select_element_length_property::define(scope, p)?;
    super::html_select_element_selected_options_property::define(scope, p)?;
    super::html_select_element_selected_index_property::define(scope, p)?;
    super::html_select_element_value_property::define(scope, p)?;
    super::html_select_element_will_validate_property::define(scope, p)?;
    super::html_select_element_validity_property::define(scope, p)?;
    super::html_select_element_validation_message_property::define(scope, p)?;
    super::html_select_element_labels_property::define(scope, p)?;
    super::html_select_element_add::define(scope, p)?;
    super::html_select_element_check_validity::define(scope, p)?;
    super::html_select_element_item::define(scope, p)?;
    super::html_select_element_named_item::define(scope, p)?;
    super::html_select_element_remove::define(scope, p)?;
    super::html_select_element_report_validity::define(scope, p)?;
    super::html_select_element_set_custom_validity::define(scope, p)?;
    super::html_select_element_show_picker::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_indexed_iterator(scope, p)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .ok_or_else(|| "HTMLSelectElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create HTMLSelectElement".to_owned());
    }
    super::html_element::attach(scope, o, "SELECT");
    let options = super::html_options_collection::create(scope, o)?;
    let selected_options = super::html_collection::create(scope, Vec::new())?;
    let validity =
        super::validity_state::create(scope, super::validity_state::ValidityRecord::default())?;
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, o);
    let options = v8::Global::new(scope, options);
    let selected_options = v8::Global::new(scope, selected_options);
    let validity = v8::Global::new(scope, validity);
    let labels = v8::Global::new(scope, labels);
    scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .ok_or_else(|| "HTMLSelectElement state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            SelectRecord {
                autocomplete: String::new(),
                disabled: false,
                multiple: false,
                name: String::new(),
                required: false,
                size: 0,
                custom_validity: String::new(),
                options,
                selected_options,
                validity,
                labels,
                picker_open: false,
                selection_explicit: false,
            },
        );
    Ok(o)
}
pub(crate) fn reset_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if record(scope, object).is_none() {
        return false;
    }
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&object.get_identity_hash().get()))
    {
        x.selection_explicit = false;
    }
    for option in options_snapshot(scope, object) {
        let _ = super::html_option_element::reset_selected(scope, option);
    }
    refresh(scope, object);
    true
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<SelectRecord> {
    scope
        .get_slot::<HtmlSelectElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn is_select(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}
pub(crate) fn options_snapshot<'s>(
    scope: &v8::PinScope<'s, '_>,
    select: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let mut options = Vec::new();
    for child in super::node::children(scope, select) {
        if super::html_option_element::is_option(scope, child) {
            options.push(child);
        } else if super::html_opt_group_element::is_opt_group(scope, child) {
            for nested in super::node::children(scope, child) {
                if super::html_option_element::is_option(scope, nested) {
                    options.push(nested);
                }
            }
        }
    }
    options
}
pub(crate) fn selected_index(
    scope: &v8::PinScope<'_, '_>,
    select: v8::Local<'_, v8::Object>,
) -> i32 {
    options_snapshot(scope, select)
        .iter()
        .position(|o| super::html_option_element::option_selected(scope, *o).unwrap_or(false))
        .map_or(-1, |i| i as i32)
}
pub(crate) fn set_selected_index_value(
    scope: &mut v8::PinScope<'_, '_>,
    select: v8::Local<'_, v8::Object>,
    index: i32,
) {
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&select.get_identity_hash().get()))
    {
        x.selection_explicit = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    for (i, o) in options_snapshot(scope, select).iter().enumerate() {
        let _ = super::html_option_element::set_option_selected(scope, *o, i as i32 == index);
    }
    refresh(scope, select)
}
pub(crate) fn containing_select<'s>(
    scope: &v8::PinScope<'s, '_>,
    option: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = super::node::parent(scope, option)?;
    if is_select(scope, parent) {
        return Some(parent);
    }
    if super::html_opt_group_element::is_opt_group(scope, parent) {
        let select = super::node::parent(scope, parent)?;
        if is_select(scope, select) {
            return Some(select);
        }
    }
    None
}
pub(crate) fn option_selected_by_script(
    scope: &mut v8::PinScope<'_, '_>,
    option: v8::Local<'_, v8::Object>,
    selected: bool,
) {
    let Some(select) = containing_select(scope, option) else {
        return;
    };
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&select.get_identity_hash().get()))
    {
        x.selection_explicit = true;
    }
    let multiple = record(scope, select).is_some_and(|x| x.multiple);
    if selected && !multiple {
        for candidate in options_snapshot(scope, select) {
            if !candidate.strict_equals(option.into()) {
                let _ = super::html_option_element::set_option_selected(scope, candidate, false);
            }
        }
    }
    refresh(scope, select)
}
pub(crate) fn refresh(scope: &mut v8::PinScope<'_, '_>, select: v8::Local<'_, v8::Object>) {
    let Some(snapshot) = record(scope, select) else {
        return;
    };
    let options = options_snapshot(scope, select);
    if !snapshot.multiple
        && !snapshot.selection_explicit
        && !options.is_empty()
        && !options
            .iter()
            .any(|o| super::html_option_element::option_selected(scope, *o).unwrap_or(false))
    {
        let _ = super::html_option_element::set_option_selected(scope, options[0], true);
    }
    let options = options_snapshot(scope, select);
    let selected = options
        .iter()
        .copied()
        .filter(|o| super::html_option_element::option_selected(scope, *o).unwrap_or(false))
        .collect::<Vec<_>>();
    let options_collection = v8::Local::new(scope, &snapshot.options);
    let _ = super::html_options_collection::refresh(scope, options_collection, options);
    let selected_collection = v8::Local::new(scope, &snapshot.selected_options);
    let _ = super::html_collection::replace(scope, selected_collection, selected);
}
pub(crate) fn add_option_value(
    scope: &mut v8::PinScope<'_, '_>,
    select: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    before: v8::Local<'_, v8::Value>,
) {
    if record(scope, select).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(option) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "The element must be an option or optgroup");
        return;
    };
    if !super::html_option_element::is_option(scope, option)
        && !super::html_opt_group_element::is_opt_group(scope, option)
    {
        crate::webidl::throw_type_error(scope, "The element must be an option or optgroup");
        return;
    }
    let options = options_snapshot(scope, select);
    let index = if before.is_undefined() || before.is_null() {
        options.len()
    } else if let Some(i) = before.int32_value(scope) {
        if i < 0 {
            options.len()
        } else {
            (i as usize).min(options.len())
        }
    } else if let Ok(before) = v8::Local::<v8::Object>::try_from(before) {
        options
            .iter()
            .position(|o| o.strict_equals(before.into()))
            .unwrap_or(options.len())
    } else {
        options.len()
    };
    let raw_index = if index == options.len() {
        super::node::children(scope, select).len()
    } else {
        super::node::children(scope, select)
            .iter()
            .position(|o| o.strict_equals(options[index].into()))
            .unwrap_or(super::node::children(scope, select).len())
    };
    let _ = super::node::insert_child(scope, select, option, raw_index);
    refresh(scope, select)
}
pub(crate) fn remove_option_index(
    scope: &mut v8::PinScope<'_, '_>,
    select: v8::Local<'_, v8::Object>,
    index: i32,
) {
    let options = options_snapshot(scope, select);
    if index >= 0 && (index as usize) < options.len() {
        let _ = super::node::detach(scope, options[index as usize]);
        refresh(scope, select)
    }
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&SelectRecord) -> &str,
) {
    if let Some(x) = record(scope, a.this()) {
        if let Some(v) = v8::String::new(scope, select(&x)) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn update_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut SelectRecord, String),
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        update(x, v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn return_bool(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&SelectRecord) -> bool,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, select(&x)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn update_bool(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut SelectRecord, bool),
) {
    let v = a.get(0).boolean_value(scope);
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        update(x, v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let should_refresh = name.eq_ignore_ascii_case("multiple");
    let Some(record) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return;
    };
    let text = value.unwrap_or_default();
    match name.to_ascii_lowercase().as_str() {
        "autocomplete" => record.autocomplete = text.to_owned(),
        "disabled" => record.disabled = value.is_some(),
        "multiple" => record.multiple = value.is_some(),
        "name" => record.name = text.to_owned(),
        "required" => record.required = value.is_some(),
        "size" => record.size = text.parse().unwrap_or(0),
        _ => {}
    }
    if should_refresh {
        refresh(scope, object);
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
pub(crate) fn get_autocomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.autocomplete)
}
pub(crate) fn set_autocomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_string(s, a, |x, v| x.autocomplete = v)
}
pub(crate) fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.name)
}
pub(crate) fn set_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_string(s, a, |x, v| x.name = v)
}
pub(crate) fn get_disabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.disabled)
}
pub(crate) fn set_disabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_bool(s, a, |x, v| x.disabled = v)
}
pub(crate) fn get_multiple(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.multiple)
}
pub(crate) fn set_multiple(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let this = a.this();
    update_bool(s, a, |x, v| x.multiple = v);
    refresh(s, this)
}
pub(crate) fn get_required(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.required)
}
pub(crate) fn set_required(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_bool(s, a, |x, v| x.required = v)
}
pub(crate) fn get_form(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else if let Some(form) = super::html_form_element::ancestor_form(scope, a.this()) {
        r.set(form.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
pub(crate) fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, x.size).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_size(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.size = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(
        scope,
        if x.multiple {
            "select-multiple"
        } else {
            "select-one"
        },
    ) {
        r.set(v.into())
    }
}
pub(crate) fn get_options(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    refresh(scope, a.this());
    r.set(v8::Local::new(scope, &x.options).into())
}
pub(crate) fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(
            v8::Integer::new_from_unsigned(scope, options_snapshot(scope, a.this()).len() as u32)
                .into(),
        )
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested = a.get(0).uint32_value(scope).unwrap_or(0) as usize;
    let current = options_snapshot(scope, a.this());
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if requested < current.len() {
        for o in current[requested..].iter().rev() {
            let _ = super::node::detach(scope, *o);
        }
    } else {
        for _ in current.len()..requested {
            if let Ok(o) = super::html_option_element::create(
                scope,
                String::new(),
                String::new(),
                false,
                false,
            ) {
                let index = super::node::children(scope, a.this()).len();
                let _ = super::node::insert_child(scope, a.this(), o, index);
            }
        }
    }
    refresh(scope, a.this())
}
pub(crate) fn get_selected_options(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    refresh(scope, a.this());
    r.set(v8::Local::new(scope, &x.selected_options).into())
}
pub(crate) fn get_selected_index(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(v8::Integer::new(scope, selected_index(scope, a.this())).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_selected_index(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let i = a.get(0).int32_value(scope).unwrap_or(-1);
    set_selected_index_value(scope, a.this(), i)
}
pub(crate) fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = options_snapshot(scope, a.this())
        .into_iter()
        .find(|o| super::html_option_element::option_selected(scope, *o).unwrap_or(false))
        .and_then(|o| super::html_option_element::option_value(scope, o))
        .unwrap_or_default();
    if let Some(v) = v8::String::new(scope, &value) {
        r.set(v.into())
    }
}
pub(crate) fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.selection_explicit = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut found = false;
    for o in options_snapshot(scope, a.this()) {
        let matched = !found
            && super::html_option_element::option_value(scope, o).is_some_and(|v| v == value);
        let _ = super::html_option_element::set_option_selected(scope, o, matched);
        found |= matched;
    }
    refresh(scope, a.this())
}
pub(crate) fn invalid(scope: &v8::PinScope<'_, '_>, select: v8::Local<'_, v8::Object>) -> bool {
    record(scope, select).is_some_and(|x| {
        !x.custom_validity.is_empty() || (x.required && selected_index(scope, select) < 0)
    })
}
pub(crate) fn get_will_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, !x.disabled).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let validity = v8::Local::new(scope, &x.validity);
    let _ = super::validity_state::replace(
        scope,
        validity,
        super::validity_state::ValidityRecord {
            value_missing: x.required && selected_index(scope, a.this()) < 0,
            custom_error: !x.custom_validity.is_empty(),
            ..Default::default()
        },
    );
    r.set(validity.into())
}
pub(crate) fn get_validation_message(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let m = if !x.custom_validity.is_empty() {
        x.custom_validity
    } else if x.required && selected_index(scope, a.this()) < 0 {
        "Please select an item in the list.".to_owned()
    } else {
        String::new()
    };
    if let Some(v) = v8::String::new(scope, &m) {
        r.set(v.into())
    }
}
pub(crate) fn get_labels(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &x.labels).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn add(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    add_option_value(scope, a.this(), a.get(0), a.get(1))
}
pub(crate) fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, x.disabled || !invalid(scope, a.this())).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn report_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    check_validity(scope, a, r)
}
pub(crate) fn item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let i = a.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(o) = options_snapshot(scope, a.this()).get(i) {
        r.set((*o).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
pub(crate) fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(scope, a.get(0));
    for o in options_snapshot(scope, a.this()) {
        if super::element::record(scope, o).is_some_and(|e| {
            e.attributes.iter().any(|(n, v)| {
                (n.eq_ignore_ascii_case("id") || n.eq_ignore_ascii_case("name")) && v == &name
            })
        }) {
            r.set(o.into());
            return;
        }
    }
    r.set(v8::null(scope).into())
}
pub(crate) fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.get(0).is_undefined() {
        let _ = super::node::detach(scope, a.this());
    } else {
        let i = a.get(0).int32_value(scope).unwrap_or(-1);
        remove_option_index(scope, a.this(), i)
    }
}
pub(crate) fn set_custom_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.custom_validity = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn show_picker(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.picker_open = true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
