use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct ButtonRecord {
    pub(crate) disabled: bool,
    pub(crate) form_action: String,
    pub(crate) form_enctype: String,
    pub(crate) form_method: String,
    pub(crate) form_no_validate: bool,
    pub(crate) form_target: String,
    pub(crate) name: String,
    pub(crate) button_type: String,
    pub(crate) value: String,
    pub(crate) custom_validity: String,
    pub(crate) validity: v8::Global<v8::Object>,
    pub(crate) labels: v8::Global<v8::Object>,
    pub(crate) popover_target: Option<v8::Global<v8::Object>>,
    pub(crate) popover_target_action: String,
    pub(crate) command_for: Option<v8::Global<v8::Object>>,
    pub(crate) command: String,
    pub(crate) interest_for: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct HtmlButtonElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ButtonRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlButtonElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLButtonElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlButtonElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLButtonElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_button_element_disabled_property::define(scope, prototype)?;
    super::html_button_element_form_property::define(scope, prototype)?;
    super::html_button_element_form_action_property::define(scope, prototype)?;
    super::html_button_element_form_enctype_property::define(scope, prototype)?;
    super::html_button_element_form_method_property::define(scope, prototype)?;
    super::html_button_element_form_no_validate_property::define(scope, prototype)?;
    super::html_button_element_form_target_property::define(scope, prototype)?;
    super::html_button_element_name_property::define(scope, prototype)?;
    super::html_button_element_type_property::define(scope, prototype)?;
    super::html_button_element_value_property::define(scope, prototype)?;
    super::html_button_element_will_validate_property::define(scope, prototype)?;
    super::html_button_element_validity_property::define(scope, prototype)?;
    super::html_button_element_validation_message_property::define(scope, prototype)?;
    super::html_button_element_labels_property::define(scope, prototype)?;
    super::html_button_element_popover_target_element_property::define(scope, prototype)?;
    super::html_button_element_popover_target_action_property::define(scope, prototype)?;
    super::html_button_element_command_for_element_property::define(scope, prototype)?;
    super::html_button_element_command_property::define(scope, prototype)?;
    super::html_button_element_interest_for_element_property::define(scope, prototype)?;
    super::html_button_element_check_validity::define(scope, prototype)?;
    super::html_button_element_report_validity::define(scope, prototype)?;
    super::html_button_element_set_custom_validity::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlButtonElementStore>()
        .ok_or_else(|| "HTMLButtonElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLButtonElement".to_owned());
    }
    super::html_element::attach(scope, object, "BUTTON");
    let validity =
        super::validity_state::create(scope, super::validity_state::ValidityRecord::default())?;
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, object);
    let validity = v8::Global::new(scope, validity);
    let labels = v8::Global::new(scope, labels);
    scope
        .get_slot_mut::<HtmlButtonElementStore>()
        .ok_or_else(|| "HTMLButtonElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ButtonRecord {
                disabled: false,
                form_action: "about:blank".to_owned(),
                form_enctype: String::new(),
                form_method: String::new(),
                form_no_validate: false,
                form_target: String::new(),
                name: String::new(),
                button_type: "submit".to_owned(),
                value: String::new(),
                custom_validity: String::new(),
                validity,
                labels,
                popover_target: None,
                popover_target_action: "toggle".to_owned(),
                command_for: None,
                command: String::new(),
                interest_for: None,
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
) -> Option<ButtonRecord> {
    scope
        .get_slot::<HtmlButtonElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut ButtonRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlButtonElementStore>()
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
) {
    if record(scope, object).is_none() {
        return;
    }
    if name.eq_ignore_ascii_case("interestfor") {
        update(scope, object, |record| record.interest_for = None);
    } else if name.eq_ignore_ascii_case("popovertarget") {
        update(scope, object, |record| record.popover_target = None);
    }
}

pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ButtonRecord) -> &str,
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
    change: impl FnOnce(&mut ButtonRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, x.disabled).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |x| x.disabled = value);
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
    set_string(s, a, |x, v| {
        x.form_action = if v.starts_with('/') {
            String::new()
        } else if v.is_empty() {
            "about:blank".to_owned()
        } else {
            v
        }
    });
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
    set_string(s, a, |x, v| {
        x.form_enctype = match v.to_ascii_lowercase().as_str() {
            "multipart/form-data" => "multipart/form-data".to_owned(),
            "text/plain" => "text/plain".to_owned(),
            _ => "application/x-www-form-urlencoded".to_owned(),
        }
    });
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
    set_string(s, a, |x, v| {
        x.form_method = match v.to_ascii_lowercase().as_str() {
            "post" => "post".to_owned(),
            "dialog" => "dialog".to_owned(),
            _ => "get".to_owned(),
        }
    });
}
pub(crate) fn get_form_no_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, x.form_no_validate).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_form_no_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |x| x.form_no_validate = value);
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
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.button_type);
}
pub(crate) fn set_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0)).to_ascii_lowercase();
    let value = match value.as_str() {
        "reset" => "reset",
        "button" => "button",
        _ => "submit",
    }
    .to_owned();
    update(scope, a.this(), |x| x.button_type = value);
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
    set_string(s, a, |x, v| x.value = v);
}

pub(crate) fn is_candidate(record: &ButtonRecord) -> bool {
    !record.disabled && record.button_type == "submit"
}
pub(crate) fn get_will_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, is_candidate(&x)).into())
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
    super::validity_state::replace(
        scope,
        validity,
        super::validity_state::ValidityRecord {
            custom_error: !x.custom_validity.is_empty(),
            ..Default::default()
        },
    );
    r.set(validity.into());
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
    let value = if is_candidate(&x) {
        x.custom_validity
    } else {
        String::new()
    };
    if let Some(value) = v8::String::new(scope, &value) {
        r.set(value.into())
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

pub(crate) fn get_target(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&ButtonRecord) -> Option<v8::Global<v8::Object>>,
) {
    match record(scope, a.this()) {
        Some(x) => match select(&x) {
            Some(value) => r.set(v8::Local::new(scope, &value).into()),
            None => r.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
pub(crate) fn set_target(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut ButtonRecord) -> &mut Option<v8::Global<v8::Object>>,
) {
    let value = if a.get(0).is_null_or_undefined() {
        None
    } else if let Ok(value) = v8::Local::<v8::Object>::try_from(a.get(0)) {
        Some(v8::Global::new(scope, value))
    } else {
        crate::webidl::throw_type_error(scope, "The target must be an Element or null");
        return;
    };
    if let Some(x) = scope
        .get_slot_mut::<HtmlButtonElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        *select(x) = value
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_popover_target_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_target(s, a, r, |x| x.popover_target.clone())
}
pub(crate) fn set_popover_target_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_target(s, a, |x| &mut x.popover_target)
}
pub(crate) fn get_popover_target_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.popover_target_action)
}
pub(crate) fn set_popover_target_action(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0)).to_ascii_lowercase();
    let value = match value.as_str() {
        "show" => "show",
        "hide" => "hide",
        _ => "toggle",
    }
    .to_owned();
    update(scope, a.this(), |x| x.popover_target_action = value)
}
pub(crate) fn get_command_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_target(s, a, r, |x| x.command_for.clone())
}
pub(crate) fn set_command_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_target(s, a, |x| &mut x.command_for)
}
pub(crate) fn get_command(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.command)
}
pub(crate) fn set_command(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    let valid = value.starts_with("--")
        || matches!(
            value.as_str(),
            "show-popover"
                | "hide-popover"
                | "toggle-popover"
                | "show-modal"
                | "close"
                | "request-close"
        );
    update(scope, a.this(), |x| {
        x.command = if valid { value } else { String::new() }
    })
}
pub(crate) fn get_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_target(s, a, r, |x| x.interest_for.clone())
}
pub(crate) fn set_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_target(s, a, |x| &mut x.interest_for)
}
pub(crate) fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, !is_candidate(&x) || x.custom_validity.is_empty()).into())
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
pub(crate) fn set_custom_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    update(scope, a.this(), |x| x.custom_validity = value)
}
