use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct FieldSetRecord {
    pub(crate) disabled: bool,
    pub(crate) name: String,
    pub(crate) elements: v8::Global<v8::Object>,
    pub(crate) validity: v8::Global<v8::Object>,
    pub(crate) custom_validity: String,
}

#[derive(Default)]
pub(crate) struct HtmlFieldSetElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, FieldSetRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlFieldSetElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLFieldSetElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlFieldSetElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLFieldSetElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_field_set_element_disabled_property::define(scope, prototype)?;
    super::html_field_set_element_form_property::define(scope, prototype)?;
    super::html_field_set_element_name_property::define(scope, prototype)?;
    super::html_field_set_element_type_property::define(scope, prototype)?;
    super::html_field_set_element_elements_property::define(scope, prototype)?;
    super::html_field_set_element_will_validate_property::define(scope, prototype)?;
    super::html_field_set_element_validity_property::define(scope, prototype)?;
    super::html_field_set_element_validation_message_property::define(scope, prototype)?;
    super::html_field_set_element_check_validity::define(scope, prototype)?;
    super::html_field_set_element_report_validity::define(scope, prototype)?;
    super::html_field_set_element_set_custom_validity::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlFieldSetElementStore>()
        .ok_or_else(|| "HTMLFieldSetElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLFieldSetElement".to_owned());
    }
    super::html_element::attach(scope, object, "FIELDSET");
    let elements = super::html_collection::create(scope, Vec::new())?;
    let validity = super::validity_state::create(
        scope,
        super::validity_state::ValidityRecord {
            custom_error: false,
            ..Default::default()
        },
    )?;
    let elements = v8::Global::new(scope, elements);
    let validity = v8::Global::new(scope, validity);
    scope
        .get_slot_mut::<HtmlFieldSetElementStore>()
        .ok_or_else(|| "HTMLFieldSetElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            FieldSetRecord {
                disabled: false,
                name: String::new(),
                elements,
                validity,
                custom_validity: String::new(),
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
) -> Option<FieldSetRecord> {
    scope
        .get_slot::<HtmlFieldSetElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_listed_element(
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

pub(crate) fn collect_descendants<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
    output: &mut Vec<v8::Local<'s, v8::Object>>,
) {
    for child in super::node::children(scope, root) {
        if is_listed_element(scope, child) {
            output.push(child);
        }
        collect_descendants(scope, child, output);
    }
}

pub(crate) fn ancestor_form<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::html_form_element::ancestor_form(scope, object)
}

pub(crate) fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.disabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<HtmlFieldSetElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.disabled = value;
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
    } else if let Some(form) = ancestor_form(scope, arguments.this()) {
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
        .get_slot_mut::<HtmlFieldSetElementStore>()
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
        if let Some(value) = v8::String::new(scope, "fieldset") {
            result.set(value.into());
        }
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
    let mut items = Vec::new();
    collect_descendants(scope, arguments.this(), &mut items);
    let collection = v8::Local::new(scope, &record.elements);
    super::html_collection::replace(scope, collection, items);
    result.set(collection.into());
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
    if let Some(record) = record(scope, arguments.this()) {
        let validity = v8::Local::new(scope, &record.validity);
        super::validity_state::replace(
            scope,
            validity,
            super::validity_state::ValidityRecord {
                custom_error: !record.custom_validity.is_empty(),
                ..Default::default()
            },
        );
        result.set(validity.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
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
        .get_slot_mut::<HtmlFieldSetElementStore>()
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
