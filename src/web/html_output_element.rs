use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct OutputRecord {
    pub(crate) html_for: v8::Global<v8::Object>,
    pub(crate) name: String,
    pub(crate) default_value: String,
    pub(crate) value: String,
    pub(crate) value_dirty: bool,
    pub(crate) custom_validity: String,
    pub(crate) validity: v8::Global<v8::Object>,
    pub(crate) labels: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct HtmlOutputElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, OutputRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlOutputElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLOutputElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlOutputElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLOutputElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_output_element_html_for_property::define(scope, prototype)?;
    super::html_output_element_form_property::define(scope, prototype)?;
    super::html_output_element_name_property::define(scope, prototype)?;
    super::html_output_element_type_property::define(scope, prototype)?;
    super::html_output_element_default_value_property::define(scope, prototype)?;
    super::html_output_element_value_property::define(scope, prototype)?;
    super::html_output_element_will_validate_property::define(scope, prototype)?;
    super::html_output_element_validity_property::define(scope, prototype)?;
    super::html_output_element_validation_message_property::define(scope, prototype)?;
    super::html_output_element_labels_property::define(scope, prototype)?;
    super::html_output_element_check_validity::define(scope, prototype)?;
    super::html_output_element_report_validity::define(scope, prototype)?;
    super::html_output_element_set_custom_validity::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .ok_or_else(|| "HTMLOutputElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLOutputElement".to_owned());
    }
    super::html_element::attach(scope, object, "OUTPUT");
    let html_for = super::dom_token_list::create(scope, "")?;
    let validity =
        super::validity_state::create(scope, super::validity_state::ValidityRecord::default())?;
    let labels = super::node_list::create(scope, Vec::new())?;
    super::node_list::register_labels_owner(scope, labels, object);
    let record = OutputRecord {
        html_for: v8::Global::new(scope, html_for),
        name: String::new(),
        default_value: String::new(),
        value: String::new(),
        value_dirty: false,
        custom_validity: String::new(),
        validity: v8::Global::new(scope, validity),
        labels: v8::Global::new(scope, labels),
    };
    scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .ok_or_else(|| "HTMLOutputElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn reset_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.value = record.default_value.clone();
        record.value_dirty = false;
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
) -> Option<OutputRecord> {
    scope
        .get_slot::<HtmlOutputElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_html_for(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.html_for).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_html_for(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = record(scope, arguments.this()) {
        let html_for = v8::Local::new(scope, &record.html_for);
        let _ = super::dom_token_list::set_string_value(scope, html_for, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
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

pub(crate) fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.name) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.name = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        if let Some(value) = v8::String::new(scope, "output") {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_default_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.default_value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_default_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.default_value = value.clone();
        if !record.value_dirty {
            record.value = value;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.value = value;
        record.value_dirty = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_will_validate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, false).into());
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
    let _ = super::validity_state::replace(
        scope,
        validity,
        super::validity_state::ValidityRecord {
            custom_error: !record.custom_validity.is_empty(),
            ..Default::default()
        },
    );
    result.set(validity.into());
}

pub(crate) fn get_validation_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        if let Some(value) = v8::String::new(scope, "") {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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

pub(crate) fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, true).into());
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

pub(crate) fn set_custom_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlOutputElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.custom_validity = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
