use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct FormRecord {
    pub(crate) accept_charset: String,
    pub(crate) action: String,
    pub(crate) autocomplete: String,
    pub(crate) enctype: String,
    pub(crate) method: String,
    pub(crate) name: String,
    pub(crate) no_validate: bool,
    pub(crate) target: String,
    pub(crate) rel_list: v8::Global<v8::Object>,
    pub(crate) elements: v8::Global<v8::Object>,
    pub(crate) submit_count: u64,
    pub(crate) reset_count: u64,
}

#[derive(Default)]
pub(crate) struct HtmlFormElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, FormRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlFormElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLFormElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlFormElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLFormElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_form_element_accept_charset_property::define(scope, prototype)?;
    super::html_form_element_action_property::define(scope, prototype)?;
    super::html_form_element_autocomplete_property::define(scope, prototype)?;
    super::html_form_element_enctype_property::define(scope, prototype)?;
    super::html_form_element_encoding_property::define(scope, prototype)?;
    super::html_form_element_method_property::define(scope, prototype)?;
    super::html_form_element_name_property::define(scope, prototype)?;
    super::html_form_element_no_validate_property::define(scope, prototype)?;
    super::html_form_element_target_property::define(scope, prototype)?;
    super::html_form_element_rel_property::define(scope, prototype)?;
    super::html_form_element_rel_list_property::define(scope, prototype)?;
    super::html_form_element_elements_property::define(scope, prototype)?;
    super::html_form_element_length_property::define(scope, prototype)?;
    super::html_form_element_check_validity::define(scope, prototype)?;
    super::html_form_element_report_validity::define(scope, prototype)?;
    super::html_form_element_request_submit::define(scope, prototype)?;
    super::html_form_element_reset::define(scope, prototype)?;
    super::html_form_element_submit::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlFormElementStore>()
        .ok_or_else(|| "HTMLFormElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLFormElement".to_owned());
    }
    super::html_element::attach(scope, object, "FORM");
    let rel_list = super::dom_token_list::create_bound(scope, "", object, "rel")?;
    let elements = super::html_form_controls_collection::create(scope, Vec::new())?;
    super::html_collection::register_form_owner(scope, elements, object);
    let rel_list = v8::Global::new(scope, rel_list);
    let elements = v8::Global::new(scope, elements);
    scope
        .get_slot_mut::<HtmlFormElementStore>()
        .ok_or_else(|| "HTMLFormElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            FormRecord {
                accept_charset: String::new(),
                action: "about:blank".to_owned(),
                autocomplete: "on".to_owned(),
                enctype: "application/x-www-form-urlencoded".to_owned(),
                method: "get".to_owned(),
                name: String::new(),
                no_validate: false,
                target: String::new(),
                rel_list,
                elements,
                submit_count: 0,
                reset_count: 0,
            },
        );
    Ok(object)
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
) -> Option<FormRecord> {
    scope
        .get_slot::<HtmlFormElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_form(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<HtmlFormElementStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let Some(record) = scope
        .get_slot_mut::<HtmlFormElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return;
    };
    let text = value.unwrap_or_default();
    match name.to_ascii_lowercase().as_str() {
        "accept-charset" => record.accept_charset = text.to_owned(),
        "action" => record.action = text.to_owned(),
        "autocomplete" => {
            record.autocomplete = if text.eq_ignore_ascii_case("off") {
                "off"
            } else {
                "on"
            }
            .to_owned()
        }
        "enctype" => {
            record.enctype = match text.to_ascii_lowercase().as_str() {
                "multipart/form-data" => "multipart/form-data",
                "text/plain" => "text/plain",
                _ => "application/x-www-form-urlencoded",
            }
            .to_owned()
        }
        "method" => {
            record.method = match text.to_ascii_lowercase().as_str() {
                "post" => "post",
                "dialog" => "dialog",
                _ => "get",
            }
            .to_owned()
        }
        "name" => record.name = text.to_owned(),
        "novalidate" => record.no_validate = value.is_some(),
        "target" => record.target = text.to_owned(),
        _ => {}
    }
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut FormRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlFormElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn is_listed_control(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    super::element::record(scope, object).is_some_and(|element| {
        matches!(
            element.tag_name.as_str(),
            "BUTTON" | "FIELDSET" | "INPUT" | "OBJECT" | "OUTPUT" | "SELECT" | "TEXTAREA"
        )
    })
}

pub(crate) fn collect_controls_into<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    output: &mut Vec<v8::Local<'s, v8::Object>>,
) {
    for child in super::node::children(scope, root) {
        let is_nested_form =
            super::element::record(scope, child).is_some_and(|element| element.tag_name == "FORM");
        if is_nested_form {
            continue;
        }
        if is_listed_control(scope, child) {
            output.push(child);
        }
        collect_controls_into(scope, child, output);
    }
}

pub(crate) fn collect_controls<'s>(
    scope: &v8::PinScope<'s, '_>,
    form: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let candidates = if super::node::is_connected(scope, form) {
        super::node::owner_document(scope, form)
            .map(|document| super::document::document_descendants(scope, document))
            .unwrap_or_default()
    } else {
        let mut root = form;
        while let Some(parent) = super::node::parent(scope, root) {
            root = parent;
        }
        let mut candidates = super::dom_selector::descendants(scope, root);
        candidates.insert(0, root);
        candidates
    };
    candidates
        .into_iter()
        .filter(|candidate| is_listed_control(scope, *candidate))
        .filter(|candidate| {
            ancestor_form(scope, *candidate).is_some_and(|owner| owner.strict_equals(form.into()))
        })
        .collect()
}

pub(crate) fn ancestor_form<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    if super::element::record(scope, object).is_none() {
        return None;
    }
    if let Some(form_id) = super::element::attribute_value(scope, object, "form") {
        if form_id.is_empty() {
            return None;
        }
        let candidates = if super::node::is_connected(scope, object) {
            let document = super::node::owner_document(scope, object)?;
            super::document::document_descendants(scope, document)
        } else {
            let mut root = object;
            while let Some(parent) = super::node::parent(scope, root) {
                root = parent;
            }
            let mut candidates = super::dom_selector::descendants(scope, root);
            candidates.insert(0, root);
            candidates
        };
        return candidates.into_iter().find(|candidate| {
            is_form(scope, *candidate)
                && super::element::attribute_value(scope, *candidate, "id").as_deref()
                    == Some(form_id.as_str())
        });
    }
    let mut current = object;
    while let Some(parent) = super::node::parent(scope, current) {
        if record(scope, parent).is_some() {
            return Some(parent);
        }
        current = parent;
    }
    None
}

pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&FormRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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

pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut FormRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_accept_charset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.accept_charset);
}
pub(crate) fn set_accept_charset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.accept_charset = v);
}
pub(crate) fn get_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::resolved_url_attribute(s, a.this(), "action")
        .unwrap_or_else(|| super::element::element_base_url(s, a.this()));
    if let Some(value) = v8::String::new(s, &value) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "action", value);
}
pub(crate) fn get_autocomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.autocomplete);
}
pub(crate) fn set_autocomplete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    let normalized = if value == "off" { "off" } else { "on" }.to_owned();
    update(scope, arguments.this(), |record| {
        record.autocomplete = normalized
    });
}
pub(crate) fn get_enctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.enctype);
}
pub(crate) fn set_enctype(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    let normalized = match value.as_str() {
        "multipart/form-data" => "multipart/form-data",
        "text/plain" => "text/plain",
        _ => "application/x-www-form-urlencoded",
    }
    .to_owned();
    update(scope, arguments.this(), |record| {
        record.enctype = normalized
    });
}
pub(crate) fn get_encoding(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_enctype(s, a, r);
}
pub(crate) fn set_encoding(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_enctype(s, a, r);
}
pub(crate) fn get_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.method);
}
pub(crate) fn set_method(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    let normalized = match value.as_str() {
        "post" => "post",
        "dialog" => "dialog",
        _ => "get",
    }
    .to_owned();
    update(scope, arguments.this(), |record| record.method = normalized);
}
pub(crate) fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.name);
}
pub(crate) fn set_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.name = v);
}
pub(crate) fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.target);
}
pub(crate) fn set_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.target = v);
}

pub(crate) fn get_no_validate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.no_validate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_no_validate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.no_validate = value);
}

pub(crate) fn get_rel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let rel_list = v8::Local::new(scope, &record.rel_list);
    let value = super::dom_token_list::string_value(scope, rel_list).unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn set_rel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = record(scope, arguments.this()) {
        let rel_list = v8::Local::new(scope, &record.rel_list);
        super::dom_token_list::set_string_value(scope, rel_list, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_rel_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.rel_list).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_rel_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = record(scope, arguments.this()) {
        let rel_list = v8::Local::new(scope, &record.rel_list);
        super::dom_token_list::set_string_value(scope, rel_list, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_elements(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let controls = collect_controls(scope, arguments.this());
    let collection = v8::Local::new(scope, &record.elements);
    super::html_form_controls_collection::replace(scope, collection, controls);
    result.set(collection.into());
}

pub(crate) fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let controls = collect_controls(scope, arguments.this());
    let length = controls.len();
    let collection = v8::Local::new(scope, &record.elements);
    super::html_form_controls_collection::replace(scope, collection, controls);
    result.set(v8::Integer::new_from_unsigned(scope, length as u32).into());
}

pub(crate) fn controls_valid(
    scope: &mut v8::PinScope<'_, '_>,
    form: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(key) = v8::String::new(scope, "checkValidity") else {
        return false;
    };
    for control in collect_controls(scope, form) {
        let Some(callback) = control.get(scope, key.into()) else {
            continue;
        };
        let Ok(callback) = v8::Local::<v8::Function>::try_from(callback) else {
            continue;
        };
        if callback
            .call(scope, control.into(), &[])
            .is_some_and(|value| !value.boolean_value(scope))
        {
            return false;
        }
    }
    true
}

pub(crate) fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let valid = controls_valid(scope, arguments.this());
        result.set(v8::Boolean::new(scope, valid).into());
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

pub(crate) fn request_submit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !current.no_validate && !controls_valid(scope, arguments.this()) {
        return;
    }
    let event = super::event_target::create_event(scope, "submit");
    if super::event_target::dispatch(scope, arguments.this(), event) {
        update(scope, arguments.this(), |record| record.submit_count += 1);
    }
}

pub(crate) fn reset_control(scope: &mut v8::PinScope<'_, '_>, control: v8::Local<'_, v8::Object>) {
    if super::html_input_element::reset_state(scope, control) {
        return;
    }
    if super::html_select_element::reset_state(scope, control) {
        return;
    }
    if super::html_text_area_element::reset_state(scope, control) {
        return;
    }
    super::html_output_element::reset_state(scope, control);
}

pub(crate) fn reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event = super::event_target::create_event(scope, "reset");
    if !super::event_target::dispatch(scope, arguments.this(), event) {
        return;
    }
    for control in collect_controls(scope, arguments.this()) {
        reset_control(scope, control);
    }
    update(scope, arguments.this(), |record| record.reset_count += 1);
}

pub(crate) fn submit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        update(scope, arguments.this(), |record| record.submit_count += 1);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
