use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct HtmlGeolocationElementRecord {
    pub(crate) on_location: Option<v8::Global<v8::Value>>,
    pub(crate) position: Option<v8::Global<v8::Object>>,
    pub(crate) error: Option<v8::Global<v8::Object>>,
    pub(crate) accuracy_mode: String,
    pub(crate) autolocate: bool,
    pub(crate) watch: bool,
    pub(crate) on_prompt_action: Option<v8::Global<v8::Value>>,
    pub(crate) on_prompt_dismiss: Option<v8::Global<v8::Value>>,
    pub(crate) on_validation_status_change: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct HtmlGeolocationElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, HtmlGeolocationElementRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlGeolocationElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLGeolocationElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlGeolocationElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLGeolocationElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_geolocation_element_onlocation_property::define(scope, prototype)?;
    super::html_geolocation_element_position_property::define(scope, prototype)?;
    super::html_geolocation_element_error_property::define(scope, prototype)?;
    super::html_geolocation_element_accuracymode_property::define(scope, prototype)?;
    super::html_geolocation_element_autolocate_property::define(scope, prototype)?;
    super::html_geolocation_element_watch_property::define(scope, prototype)?;
    super::html_geolocation_element_is_valid_property::define(scope, prototype)?;
    super::html_geolocation_element_invalid_reason_property::define(scope, prototype)?;
    super::html_geolocation_element_initial_permission_status_property::define(scope, prototype)?;
    super::html_geolocation_element_permission_status_property::define(scope, prototype)?;
    super::html_geolocation_element_onpromptaction_property::define(scope, prototype)?;
    super::html_geolocation_element_onpromptdismiss_property::define(scope, prototype)?;
    super::html_geolocation_element_onvalidationstatuschange_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::html_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlGeolocationElementStore>()
        .ok_or_else(|| "HTMLGeolocationElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let element = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, element, prototype.into()) != Some(true) {
        return Err("cannot create HTMLGeolocationElement".to_owned());
    }
    super::html_element::attach(scope, element, "GEOLOCATION");
    scope
        .get_slot_mut::<HtmlGeolocationElementStore>()
        .ok_or_else(|| "HTMLGeolocationElement state was not prepared".to_owned())?
        .records
        .insert(
            element.get_identity_hash().get(),
            HtmlGeolocationElementRecord::default(),
        );
    Ok(element)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<HtmlGeolocationElementRecord> {
    scope
        .get_slot::<HtmlGeolocationElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut HtmlGeolocationElementRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlGeolocationElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'HTMLGeolocationElement': Illegal constructor",
    )
}

pub(crate) fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
    select: impl FnOnce(HtmlGeolocationElementRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, select(record), result)
}
pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut HtmlGeolocationElementRecord, Option<v8::Global<v8::Value>>),
) {
    let value = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    update(scope, arguments.this(), |record| change(record, value))
}
pub(crate) fn get_on_location(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.on_location)
}
pub(crate) fn set_on_location(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.on_location = v)
}
pub(crate) fn get_on_prompt_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.on_prompt_action)
}
pub(crate) fn set_on_prompt_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.on_prompt_action = v)
}
pub(crate) fn get_on_prompt_dismiss(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.on_prompt_dismiss)
}
pub(crate) fn set_on_prompt_dismiss(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.on_prompt_dismiss = v)
}
pub(crate) fn get_on_validation_status_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.on_validation_status_change)
}
pub(crate) fn set_on_validation_status_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.on_validation_status_change = v)
}

pub(crate) fn return_optional(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Object>>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Some(v) => result.set(v8::Local::new(scope, &v).into()),
        None => result.set(v8::null(scope).into()),
    }
}
pub(crate) fn get_position(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    return_optional(s, x.position, r)
}
pub(crate) fn get_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    return_optional(s, x.error, r)
}
pub(crate) fn get_accuracy_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(s, &x.accuracy_mode) {
        r.set(v.into())
    }
}
pub(crate) fn set_accuracy_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| x.accuracy_mode = v)
}
pub(crate) fn return_bool(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(&HtmlGeolocationElementRecord) -> bool,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    r.set(v8::Boolean::new(s, f(&x)).into())
}
pub(crate) fn get_autolocate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.autolocate)
}
pub(crate) fn set_autolocate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    update(s, a.this(), |x| x.autolocate = v)
}
pub(crate) fn get_watch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.watch)
}
pub(crate) fn set_watch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    update(s, a.this(), |x| x.watch = v)
}
pub(crate) fn get_is_valid(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |_| false)
}
pub(crate) fn return_text(s: &mut v8::PinScope<'_, '_>, text: &str, mut r: v8::ReturnValue<'_>) {
    if let Some(v) = v8::String::new(s, text) {
        r.set(v.into())
    }
}
pub(crate) fn get_invalid_reason(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        return_text(s, "", r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_initial_permission_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        return_text(s, "prompt", r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_permission_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        return_text(s, "prompt", r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
