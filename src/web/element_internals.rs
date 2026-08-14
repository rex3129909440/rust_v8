use std::collections::HashMap;

#[derive(Clone)]
struct ElementInternalsRecord {
    target: v8::Global<v8::Object>,
    form: Option<v8::Global<v8::Object>>,
    will_validate: bool,
    validity: v8::Global<v8::Object>,
    validation_message: String,
    labels: v8::Global<v8::Object>,
    states: v8::Global<v8::Object>,
    shadow_root: Option<v8::Global<v8::Object>>,
    strings: HashMap<String, String>,
    relations: HashMap<String, v8::Global<v8::Value>>,
    form_value: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct ElementInternalsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ElementInternalsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ElementInternalsStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ElementInternals", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<ElementInternalsStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "ElementInternals",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "form", get_form)?;
    crate::webidl::define_readonly_accessor(s, p, "willValidate", get_will_validate)?;
    crate::webidl::define_readonly_accessor(s, p, "validity", get_validity)?;
    crate::webidl::define_readonly_accessor(s, p, "validationMessage", get_validation_message)?;
    crate::webidl::define_readonly_accessor(s, p, "labels", get_labels)?;
    crate::webidl::define_readonly_accessor(s, p, "states", get_states)?;
    crate::webidl::define_readonly_accessor(s, p, "shadowRoot", get_shadow_root)?;
    define_string_accessor(s, p, "role")?;
    define_string_accessor(s, p, "ariaAtomic")?;
    define_string_accessor(s, p, "ariaAutoComplete")?;
    define_string_accessor(s, p, "ariaBusy")?;
    define_string_accessor(s, p, "ariaBrailleLabel")?;
    define_string_accessor(s, p, "ariaBrailleRoleDescription")?;
    define_string_accessor(s, p, "ariaChecked")?;
    define_string_accessor(s, p, "ariaColCount")?;
    define_string_accessor(s, p, "ariaColIndex")?;
    define_string_accessor(s, p, "ariaColSpan")?;
    define_string_accessor(s, p, "ariaCurrent")?;
    define_string_accessor(s, p, "ariaDescription")?;
    define_string_accessor(s, p, "ariaDisabled")?;
    define_string_accessor(s, p, "ariaExpanded")?;
    define_string_accessor(s, p, "ariaHasPopup")?;
    define_string_accessor(s, p, "ariaHidden")?;
    define_string_accessor(s, p, "ariaInvalid")?;
    define_string_accessor(s, p, "ariaKeyShortcuts")?;
    define_string_accessor(s, p, "ariaLabel")?;
    define_string_accessor(s, p, "ariaLevel")?;
    define_string_accessor(s, p, "ariaLive")?;
    define_string_accessor(s, p, "ariaModal")?;
    define_string_accessor(s, p, "ariaMultiLine")?;
    define_string_accessor(s, p, "ariaMultiSelectable")?;
    define_string_accessor(s, p, "ariaOrientation")?;
    define_string_accessor(s, p, "ariaPlaceholder")?;
    define_string_accessor(s, p, "ariaPosInSet")?;
    define_string_accessor(s, p, "ariaPressed")?;
    define_string_accessor(s, p, "ariaReadOnly")?;
    define_string_accessor(s, p, "ariaRelevant")?;
    define_string_accessor(s, p, "ariaRequired")?;
    define_string_accessor(s, p, "ariaRoleDescription")?;
    define_string_accessor(s, p, "ariaRowCount")?;
    define_string_accessor(s, p, "ariaRowIndex")?;
    define_string_accessor(s, p, "ariaRowSpan")?;
    define_string_accessor(s, p, "ariaSelected")?;
    define_string_accessor(s, p, "ariaSetSize")?;
    define_string_accessor(s, p, "ariaSort")?;
    define_string_accessor(s, p, "ariaValueMax")?;
    define_string_accessor(s, p, "ariaValueMin")?;
    define_string_accessor(s, p, "ariaValueNow")?;
    define_string_accessor(s, p, "ariaValueText")?;
    crate::webidl::define_method(s, p, "checkValidity", 0, check_validity)?;
    crate::webidl::define_method(s, p, "reportValidity", 0, report_validity)?;
    crate::webidl::define_method(s, p, "setFormValue", 1, set_form_value)?;
    crate::webidl::define_method(s, p, "setValidity", 1, set_validity)?;
    define_string_accessor(s, p, "ariaColIndexText")?;
    define_string_accessor(s, p, "ariaRowIndexText")?;
    define_relation_accessor(s, p, "ariaActionsElements", true)?;
    define_relation_accessor(s, p, "ariaActiveDescendantElement", false)?;
    define_relation_accessor(s, p, "ariaControlsElements", true)?;
    define_relation_accessor(s, p, "ariaDescribedByElements", true)?;
    define_relation_accessor(s, p, "ariaDetailsElements", true)?;
    define_relation_accessor(s, p, "ariaErrorMessageElements", true)?;
    define_relation_accessor(s, p, "ariaFlowToElements", true)?;
    define_relation_accessor(s, p, "ariaLabelledByElements", true)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<ElementInternalsStore>()
        .ok_or_else(|| "ElementInternals state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'_, v8::Object>,
    form: Option<v8::Local<'_, v8::Object>>,
    shadow_root: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create ElementInternals".to_owned());
    }
    let validity =
        super::validity_state::create(s, super::validity_state::ValidityRecord::default())?;
    let labels = super::node_list::create(s, Vec::new())?;
    let states = super::custom_state_set::create(s)?;
    let record = ElementInternalsRecord {
        target: v8::Global::new(s, target),
        form: form.map(|v| v8::Global::new(s, v)),
        will_validate: true,
        validity: v8::Global::new(s, validity),
        validation_message: String::new(),
        labels: v8::Global::new(s, labels),
        states: v8::Global::new(s, states),
        shadow_root: shadow_root.map(|v| v8::Global::new(s, v)),
        strings: HashMap::new(),
        relations: HashMap::new(),
        form_value: None,
    };
    s.get_slot_mut::<ElementInternalsStore>()
        .ok_or_else(|| "ElementInternals state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}

pub(crate) fn for_target<'s>(
    s: &v8::PinScope<'s, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let target_id = target.get_identity_hash().get();
    let (identity, _) =
        s.get_slot::<ElementInternalsStore>()?
            .records
            .iter()
            .find(|(_, record)| {
                v8::Local::new(s, &record.target).get_identity_hash().get() == target_id
            })?;
    let html = super::html_element::record(s, target)?;
    html.internals
        .map(|internals| v8::Local::new(s, &internals))
}

pub(crate) fn update_form_owner(
    s: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    form: Option<v8::Local<'_, v8::Object>>,
) {
    let Some(internals) = for_target(s, target) else {
        return;
    };
    let form = form.map(|value| v8::Global::new(s, value));
    if let Some(record) = s
        .get_slot_mut::<ElementInternalsStore>()
        .and_then(|store| store.records.get_mut(&internals.get_identity_hash().get()))
    {
        record.form = form;
    }
}

pub(crate) fn set_shadow_root_for_target(
    s: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    shadow_root: v8::Local<'_, v8::Object>,
) {
    let Some(internals) = for_target(s, target) else {
        return;
    };
    let shadow_root = v8::Global::new(s, shadow_root);
    if let Some(record) = s
        .get_slot_mut::<ElementInternalsStore>()
        .and_then(|store| store.records.get_mut(&internals.get_identity_hash().get()))
    {
        record.shadow_root = Some(shadow_root);
    }
}

pub(crate) fn form_value_for_target<'s>(
    s: &v8::PinScope<'s, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Value>> {
    let internals = for_target(s, target)?;
    record(s, internals)?
        .form_value
        .map(|value| v8::Local::new(s, &value))
}

pub(crate) fn is_valid_for_target(
    s: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    let internals = for_target(s, target)?;
    valid_record(s, internals).map(|(_, valid)| valid)
}

pub(crate) fn refresh_labels(s: &mut v8::PinScope<'_, '_>, internals: v8::Local<'_, v8::Object>) {
    let Some(record) = record(s, internals) else {
        return;
    };
    let target = v8::Local::new(s, &record.target);
    let labels = super::html_label_element::labels_for(s, target);
    let list = v8::Local::new(s, &record.labels);
    super::node_list::replace_snapshot(s, list, labels);
}
fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<ElementInternalsRecord> {
    s.get_slot::<ElementInternalsStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}

fn require_form_associated(
    s: &mut v8::PinScope<'_, '_>,
    internals: v8::Local<'_, v8::Object>,
    operation: &str,
    property: bool,
) -> Option<ElementInternalsRecord> {
    let Some(record) = record(s, internals) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return None;
    };
    let target = v8::Local::new(s, &record.target);
    if super::custom_element_registry::is_form_associated(s, target) {
        return Some(record);
    }
    let message = if property {
        format!(
            "Failed to read the '{operation}' property from 'ElementInternals': The target element is not a form-associated custom element."
        )
    } else {
        format!(
            "Failed to execute '{operation}' on 'ElementInternals': The target element is not a form-associated custom element."
        )
    };
    super::node::throw_dom_exception(s, "NotSupportedError", &message);
    None
}
fn return_object(
    s: &mut v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Object>>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        r.set(v8::Local::new(s, &value).into())
    } else {
        r.set(v8::null(s).into())
    }
}
fn get_form(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if let Some(x) = require_form_associated(s, a.this(), "form", true) {
        return_object(s, x.form, r)
    }
}
fn get_shadow_root(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        return_object(s, x.shadow_root, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_will_validate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if let Some(x) = require_form_associated(s, a.this(), "willValidate", true) {
        r.set(v8::Boolean::new(s, x.will_validate).into())
    }
}
fn get_validity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if let Some(x) = require_form_associated(s, a.this(), "validity", true) {
        r.set(v8::Local::new(s, &x.validity).into())
    }
}
fn get_validation_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if let Some(x) = require_form_associated(s, a.this(), "validationMessage", true) {
        if let Some(v) = v8::String::new(s, &x.validation_message) {
            r.set(v.into())
        }
    }
}
fn get_labels(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    } else if let Some(x) = require_form_associated(s, a.this(), "labels", true) {
        refresh_labels(s, a.this());
        r.set(v8::Local::new(s, &x.labels).into())
    }
}
fn get_states(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.states).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn data_name(s: &v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> String {
    crate::webidl::value_to_string(s, crate::trace::native_callback_data(s, a))
}
fn named_string_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = data_name(s, &a);
    if let Some(x) = record(s, a.this()) {
        if let Some(value) = x.strings.get(&name).and_then(|v| v8::String::new(s, v)) {
            r.set(value.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn named_string_set(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let name = data_name(s, &a);
    let value = a.get(0);
    let value = (!value.is_null_or_undefined()).then(|| crate::webidl::value_to_string(s, value));
    if let Some(x) = s
        .get_slot_mut::<ElementInternalsStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = value {
            x.strings.insert(name, value);
        } else {
            x.strings.remove(&name);
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn define_string_accessor(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let data = crate::webidl::string(s, name)?;
    let getter = crate::webidl::create_function_with_data(
        s,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        named_string_get,
        data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(s, p.into()) {
        crate::trace::relabel_native_function(s, getter, &format!("{owner}.get {name}"));
    }
    let data = crate::webidl::string(s, name)?;
    let setter = crate::webidl::create_function_with_data(
        s,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        named_string_set,
        data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(s, p.into()) {
        crate::trace::relabel_native_function(s, setter, &format!("{owner}.set {name}"));
    }
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(s, name)?;
    if p.define_property(s, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define {name}"))
    }
}
fn named_relation_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = data_name(s, &a);
    if let Some(x) = record(s, a.this()) {
        if let Some(value) = x.relations.get(&name) {
            r.set(v8::Local::new(s, value))
        } else if name.ends_with("Elements") {
            r.set(v8::Array::new(s, 0).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn named_relation_set(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let name = data_name(s, &a);
    let value = a.get(0);
    let value = (!value.is_null_or_undefined()).then(|| v8::Global::new(s, value));
    if let Some(x) = s
        .get_slot_mut::<ElementInternalsStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = value {
            x.relations.insert(name, value);
        } else {
            x.relations.remove(&name);
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn define_relation_accessor(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
    name: &str,
    _multiple: bool,
) -> Result<(), String> {
    let data = crate::webidl::string(s, name)?;
    let getter = crate::webidl::create_function_with_data(
        s,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        named_relation_get,
        data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(s, p.into()) {
        crate::trace::relabel_native_function(s, getter, &format!("{owner}.get {name}"));
    }
    let data = crate::webidl::string(s, name)?;
    let setter = crate::webidl::create_function_with_data(
        s,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        named_relation_set,
        data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(s, p.into()) {
        crate::trace::relabel_native_function(s, setter, &format!("{owner}.set {name}"));
    }
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(s, name)?;
    if p.define_property(s, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define {name}"))
    }
}
fn valid_record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<(ElementInternalsRecord, bool)> {
    let x = record(s, o)?;
    let validity = v8::Local::new(s, &x.validity);
    let key = v8::String::new(s, "valid")?;
    let valid = validity.get(s, key.into())?.boolean_value(s);
    Some((x, valid))
}
fn check_validity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if require_form_associated(s, a.this(), "checkValidity", false).is_none() {
        return;
    }
    if let Some((record, valid)) = valid_record(s, a.this()) {
        if !valid {
            let target = v8::Local::new(s, &record.target);
            super::html_form_element::dispatch_invalid_event(s, target);
        }
        r.set(v8::Boolean::new(s, valid).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn report_validity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if require_form_associated(s, a.this(), "reportValidity", false).is_none() {
        return;
    }
    if let Some((record, valid)) = valid_record(s, a.this()) {
        if !valid {
            super::html_form_element::dispatch_invalid_event(s, v8::Local::new(s, &record.target));
        }
        let mut r = r;
        r.set(v8::Boolean::new(s, valid).into());
    }
}
fn set_form_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if require_form_associated(s, a.this(), "setFormValue", false).is_none() {
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'setFormValue' on 'ElementInternals': 1 argument required, but only 0 present.",
        );
        return;
    }
    let raw = a.get(0);
    let converted: v8::Local<v8::Value> = if raw.is_null() {
        raw
    } else if let Ok(object) = v8::Local::<v8::Object>::try_from(raw) {
        if super::blob::byte_snapshot(s, object).is_some()
            || super::form_data::is_form_data(s, object)
        {
            raw
        } else {
            let Some(value) = crate::webidl::dom_string_with_context(
                s,
                raw,
                "Failed to execute 'setFormValue' on 'ElementInternals'",
            ) else {
                return;
            };
            let Some(value) = v8::String::new(s, &value) else {
                return;
            };
            value.into()
        }
    } else {
        let Some(value) = crate::webidl::dom_string_with_context(
            s,
            raw,
            "Failed to execute 'setFormValue' on 'ElementInternals'",
        ) else {
            return;
        };
        let Some(value) = v8::String::new(s, &value) else {
            return;
        };
        value.into()
    };
    let value = (!converted.is_null()).then(|| v8::Global::new(s, converted));
    if let Some(x) = s
        .get_slot_mut::<ElementInternalsStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.form_value = value
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn flag(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, name: &str) -> bool {
    let Some(key) = v8::String::new(s, name) else {
        return false;
    };
    o.get(s, key.into()).is_some_and(|v| v.boolean_value(s))
}
fn set_validity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if require_form_associated(s, a.this(), "setValidity", false).is_none() {
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'setValidity' on 'ElementInternals': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let init = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let validity = init
        .map(|o| super::validity_state::ValidityRecord {
            value_missing: flag(s, o, "valueMissing"),
            type_mismatch: flag(s, o, "typeMismatch"),
            pattern_mismatch: flag(s, o, "patternMismatch"),
            too_long: flag(s, o, "tooLong"),
            too_short: flag(s, o, "tooShort"),
            range_underflow: flag(s, o, "rangeUnderflow"),
            range_overflow: flag(s, o, "rangeOverflow"),
            step_mismatch: flag(s, o, "stepMismatch"),
            bad_input: flag(s, o, "badInput"),
            custom_error: flag(s, o, "customError"),
        })
        .unwrap_or_default();
    let message = if a.get(1).is_undefined() {
        String::new()
    } else {
        let Some(message) = crate::webidl::dom_string_with_context(
            s,
            a.get(1),
            "Failed to execute 'setValidity' on 'ElementInternals'",
        ) else {
            return;
        };
        message
    };
    let invalid = validity.value_missing
        || validity.type_mismatch
        || validity.pattern_mismatch
        || validity.too_long
        || validity.too_short
        || validity.range_underflow
        || validity.range_overflow
        || validity.step_mismatch
        || validity.bad_input
        || validity.custom_error;
    if invalid && message.is_empty() {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'setValidity' on 'ElementInternals': The second argument should not be empty if one or more flags in the first argument are true.",
        );
        return;
    }
    if !a.get(2).is_undefined() {
        let Ok(anchor) = v8::Local::<v8::Object>::try_from(a.get(2)) else {
            crate::webidl::throw_type_error(
                s,
                "Failed to execute 'setValidity' on 'ElementInternals': parameter 3 is not of type 'HTMLElement'.",
            );
            return;
        };
        if super::html_element::record(s, anchor).is_none() {
            crate::webidl::throw_type_error(
                s,
                "Failed to execute 'setValidity' on 'ElementInternals': parameter 3 is not of type 'HTMLElement'.",
            );
            return;
        }
    }
    let validity_object = v8::Local::new(s, &x.validity);
    let _ = super::validity_state::replace(s, validity_object, validity);
    if let Some(stored) = s
        .get_slot_mut::<ElementInternalsStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        stored.validation_message = message;
    }
}
