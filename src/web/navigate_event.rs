use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigateEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, NavigateEventRecord>,
}

#[derive(Clone)]
pub(crate) struct NavigateEventRecord {
    pub(crate) navigation_type: String,
    pub(crate) destination: v8::Global<v8::Object>,
    pub(crate) can_intercept: bool,
    pub(crate) user_initiated: bool,
    pub(crate) hash_change: bool,
    pub(crate) signal: v8::Global<v8::Object>,
    pub(crate) form_data: Option<v8::Global<v8::Value>>,
    pub(crate) download_request: Option<String>,
    pub(crate) info: v8::Global<v8::Value>,
    pub(crate) source_element: Option<v8::Global<v8::Object>>,
    pub(crate) has_ua_visual_transition: bool,
    pub(crate) trusted_navigation: bool,
    pub(crate) intercepted: bool,
    pub(crate) handlers: Vec<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigateEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigateEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigateEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "NavigateEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::navigate_event_navigation_type_property::define(scope, prototype)?;
    super::navigate_event_destination_property::define(scope, prototype)?;
    super::navigate_event_can_intercept_property::define(scope, prototype)?;
    super::navigate_event_user_initiated_property::define(scope, prototype)?;
    super::navigate_event_hash_change_property::define(scope, prototype)?;
    super::navigate_event_signal_property::define(scope, prototype)?;
    super::navigate_event_form_data_property::define(scope, prototype)?;
    super::navigate_event_download_request_property::define(scope, prototype)?;
    super::navigate_event_info_property::define(scope, prototype)?;
    super::navigate_event_source_element_property::define(scope, prototype)?;
    super::navigate_event_intercept::define(scope, prototype)?;
    super::navigate_event_scroll::define(scope, prototype)?;
    super::navigate_event_has_ua_visual_transition_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NavigateEventStore>()
        .ok_or_else(|| "NavigateEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NavigateEvent': 2 arguments required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The event initializer is required");
        return;
    };
    let Some(destination) = object_property(scope, init, "destination") else {
        crate::webidl::throw_type_error(scope, "destination must be a NavigationDestination");
        return;
    };
    if !super::navigation_destination::is_destination(scope, destination) {
        crate::webidl::throw_type_error(scope, "destination must be a NavigationDestination");
        return;
    }
    let Some(signal) = object_property(scope, init, "signal") else {
        crate::webidl::throw_type_error(scope, "signal must be an AbortSignal");
        return;
    };
    if super::abort_signal::record(scope, signal).is_none() {
        crate::webidl::throw_type_error(scope, "signal must be an AbortSignal");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let navigation_type =
        string_property(scope, init, "navigationType").unwrap_or_else(|| "push".to_owned());
    let can_intercept = boolean_property(scope, init, "canIntercept");
    let user_initiated = boolean_property(scope, init, "userInitiated");
    let hash_change = boolean_property(scope, init, "hashChange");
    let form_data = value_property(scope, init, "formData")
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| v8::Global::new(scope, value));
    let download_request = string_property(scope, init, "downloadRequest");
    let info = value_property(scope, init, "info").unwrap_or_else(|| v8::undefined(scope).into());
    let source_element = object_property(scope, init, "sourceElement");
    let has_ua_visual_transition = boolean_property(scope, init, "hasUAVisualTransition");
    let bubbles = boolean_property(scope, init, "bubbles");
    let cancelable = boolean_property(scope, init, "cancelable");
    let composed = boolean_property(scope, init, "composed");
    attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
        navigation_type,
        destination,
        can_intercept,
        user_initiated,
        hash_change,
        signal,
        form_data,
        download_request,
        info,
        source_element,
        has_ua_visual_transition,
        false,
    );
    result.set(arguments.this().into());
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    navigation_type: &str,
    destination: v8::Local<'_, v8::Object>,
    info: v8::Local<'_, v8::Value>,
    hash_change: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create NavigateEvent".to_owned());
    }
    let signal = super::abort_signal::create(scope, None)?;
    attach(
        scope,
        event,
        "navigate".to_owned(),
        false,
        true,
        false,
        navigation_type.to_owned(),
        destination,
        true,
        false,
        hash_change,
        signal,
        None,
        None,
        info,
        None,
        false,
        true,
    );
    Ok(event)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    navigation_type: String,
    destination: v8::Local<'_, v8::Object>,
    can_intercept: bool,
    user_initiated: bool,
    hash_change: bool,
    signal: v8::Local<'_, v8::Object>,
    form_data: Option<v8::Global<v8::Value>>,
    download_request: Option<String>,
    info: v8::Local<'_, v8::Value>,
    source_element: Option<v8::Local<'_, v8::Object>>,
    has_ua_visual_transition: bool,
    trusted_navigation: bool,
) {
    super::event::attach(scope, event, event_type, bubbles, cancelable, composed);
    let source_element = source_element.map(|element| v8::Global::new(scope, element));
    let record = NavigateEventRecord {
        navigation_type,
        destination: v8::Global::new(scope, destination),
        can_intercept,
        user_initiated,
        hash_change,
        signal: v8::Global::new(scope, signal),
        form_data,
        download_request,
        info: v8::Global::new(scope, info),
        source_element,
        has_ua_visual_transition,
        trusted_navigation,
        intercepted: false,
        handlers: Vec::new(),
    };
    if let Some(store) = scope.get_slot_mut::<NavigateEventStore>() {
        store
            .records
            .insert(event.get_identity_hash().get(), record);
    }
}

pub(crate) fn take_handlers(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> Vec<v8::Global<v8::Function>> {
    scope
        .get_slot_mut::<NavigateEventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
        .map(|record| std::mem::take(&mut record.handlers))
        .unwrap_or_default()
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigateEventRecord> {
    scope
        .get_slot::<NavigateEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigateEventRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

pub(crate) fn return_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigateEventRecord) -> bool,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, select(&record)).into());
}

pub(crate) fn get_navigation_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.navigation_type);
}

pub(crate) fn get_destination(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.destination).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_can_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(scope, arguments, result, |record| record.can_intercept);
}

pub(crate) fn get_user_initiated(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(scope, arguments, result, |record| record.user_initiated);
}

pub(crate) fn get_hash_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(scope, arguments, result, |record| record.hash_change);
}

pub(crate) fn get_signal(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.signal).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_form_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(form_data) = record.form_data {
        result.set(v8::Local::new(scope, &form_data));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_download_request(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(download_request) = record.download_request
        && let Some(value) = v8::String::new(scope, &download_request)
    {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_info(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.info));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_source_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(source_element) = record.source_element {
        result.set(v8::Local::new(scope, &source_element).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_has_ua_visual_transition(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(scope, arguments, result, |record| {
        record.has_ua_visual_transition
    });
}

pub(crate) fn intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(snapshot) = scope
        .get_slot::<NavigateEventStore>()
        .and_then(|store| store.records.get(&id))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !snapshot.trusted_navigation {
        throw_dom_exception(
            scope,
            "SecurityError",
            "intercept() may only be called on a trusted navigate event",
        );
        return;
    }
    if !snapshot.can_intercept {
        throw_dom_exception(
            scope,
            "SecurityError",
            "This navigation cannot be intercepted",
        );
        return;
    }
    let handler = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|options| value_property(scope, options, "handler"))
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .map(|function| v8::Global::new(scope, function));
    if let Some(record) = scope
        .get_slot_mut::<NavigateEventStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.intercepted = true;
        if let Some(handler) = handler {
            record.handlers.push(handler);
        }
    }
}

pub(crate) fn scroll(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.trusted_navigation {
        throw_dom_exception(
            scope,
            "SecurityError",
            "scroll() may only be called on a trusted navigate event",
        );
    } else if !record.intercepted {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "The navigation has not been intercepted",
        );
    }
}

pub(crate) fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    value_property(scope, object, name)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    value_property(scope, object, name)
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
}

pub(crate) fn boolean_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    value_property(scope, object, name).is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), name.to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
