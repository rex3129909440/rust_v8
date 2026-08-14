use std::collections::HashMap;

pub(crate) const NONE: i32 = 0;
pub(crate) const CAPTURING_PHASE: i32 = 1;
pub(crate) const AT_TARGET: i32 = 2;
pub(crate) const BUBBLING_PHASE: i32 = 3;

pub(crate) struct EventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) is_trusted_getters: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, EventRecord>,
}

impl Default for EventStore {
    fn default() -> Self {
        Self {
            constructors: HashMap::new(),
            is_trusted_getters: HashMap::new(),
            records: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct EventRecord {
    pub(crate) event_type: String,
    pub(crate) target: Option<v8::Global<v8::Object>>,
    pub(crate) current_target: Option<v8::Global<v8::Object>>,
    pub(crate) event_phase: i32,
    pub(crate) bubbles: bool,
    pub(crate) cancelable: bool,
    pub(crate) default_prevented: bool,
    pub(crate) composed: bool,
    pub(crate) cancel_bubble: bool,
    pub(crate) immediate_stopped: bool,
    pub(crate) dispatching: bool,
    pub(crate) in_passive_listener: bool,
    pub(crate) initialized: bool,
    pub(crate) time_stamp: f64,
    pub(crate) is_trusted: bool,
    pub(crate) path: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EventStore::default());
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<EventStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Event",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::event_type_property::define(scope, prototype)?;
    super::event_target_property::define(scope, prototype)?;
    super::event_current_target_property::define(scope, prototype)?;
    super::event_event_phase_property::define(scope, prototype)?;
    super::event_bubbles_property::define(scope, prototype)?;
    super::event_cancelable_property::define(scope, prototype)?;
    super::event_default_prevented_property::define(scope, prototype)?;
    super::event_composed_property::define(scope, prototype)?;
    super::event_time_stamp_property::define(scope, prototype)?;
    super::event_src_element_property::define(scope, prototype)?;
    super::event_return_value_property::define(scope, prototype)?;
    super::event_cancel_bubble_property::define(scope, prototype)?;
    define_event_constants(scope, prototype)?;
    super::event_composed_path::define(scope, prototype)?;
    super::event_init_event::define(scope, prototype)?;
    super::event_prevent_default::define(scope, prototype)?;
    super::event_stop_immediate_propagation::define(scope, prototype)?;
    super::event_stop_propagation::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_event_constants(scope, constructor.into())?;
    let is_trusted_getter = crate::webidl::create_function(
        scope,
        "get isTrusted",
        0,
        v8::ConstructorBehavior::Throw,
        get_is_trusted,
    )?;
    let constructor_global = v8::Global::new(scope, constructor);
    let is_trusted_getter_global = v8::Global::new(scope, is_trusted_getter);
    let realm_id = crate::webidl::realm_id(scope);
    let store = scope
        .get_slot_mut::<EventStore>()
        .ok_or_else(|| "Event state was not prepared".to_owned())?;
    store.constructors.insert(realm_id, constructor_global);
    store
        .is_trusted_getters
        .insert(realm_id, is_trusted_getter_global);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Event", constructor.into())
}

pub(crate) fn define_event_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "NONE", NONE)?;
    crate::webidl::define_constant(scope, object, "CAPTURING_PHASE", CAPTURING_PHASE)?;
    crate::webidl::define_constant(scope, object, "AT_TARGET", AT_TARGET)?;
    crate::webidl::define_constant(scope, object, "BUBBLING_PHASE", BUBBLING_PHASE)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'Event': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Event': 1 argument required");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let (bubbles, cancelable, composed) = event_init(scope, arguments.get(1));
    attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Event".to_owned());
    }
    attach(scope, object, event_type.to_owned(), false, false, false);
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
) {
    let time_stamp = super::performance::now_for_current_realm(scope).unwrap_or_else(|| {
        crate::determinism::relative_high_resolution_milliseconds(
            scope,
            crate::determinism::elapsed_milliseconds(scope),
            0.0,
        )
    });
    let is_trusted_getter = scope
        .get_slot::<EventStore>()
        .and_then(|store| {
            store
                .is_trusted_getters
                .get(&crate::webidl::realm_id(scope))
        })
        .cloned();
    if let Some(store) = scope.get_slot_mut::<EventStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            EventRecord {
                event_type,
                target: None,
                current_target: None,
                event_phase: NONE,
                bubbles,
                cancelable,
                default_prevented: false,
                composed,
                cancel_bubble: false,
                immediate_stopped: false,
                dispatching: false,
                in_passive_listener: false,
                initialized: true,
                time_stamp,
                is_trusted: false,
                path: Vec::new(),
            },
        );
    }
    if let Some(getter) = is_trusted_getter {
        let getter = v8::Local::new(scope, &getter);
        let setter = v8::undefined(scope);
        let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
        descriptor.set_enumerable(true);
        descriptor.set_configurable(false);
        if let Some(key) = v8::String::new(scope, "isTrusted") {
            let _ = object.define_property(scope, key.into(), &descriptor);
        }
    }
}

pub(crate) fn reinitialize(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
) {
    attach(scope, object, event_type, bubbles, cancelable, composed);
}

pub(crate) fn finish_dispatch(scope: &mut v8::PinScope<'_, '_>, event: v8::Local<'_, v8::Object>) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.current_target = None;
        record.event_phase = NONE;
        record.dispatching = false;
        record.in_passive_listener = false;
        record.cancel_bubble = false;
        record.immediate_stopped = false;
    }
}

pub(crate) fn is_event(scope: &v8::PinScope<'_, '_>, event: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<EventStore>()
        .is_some_and(|store| store.records.contains_key(&event.get_identity_hash().get()))
}

pub(crate) fn event_type(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, event).map(|record| record.event_type)
}

pub(crate) fn is_dispatching(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, event).is_some_and(|record| record.dispatching)
}

pub(crate) fn begin_dispatch(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    target: v8::Local<'_, v8::Object>,
    path: &[v8::Global<v8::Object>],
) -> bool {
    let target = v8::Global::new(scope, target);
    let path = path.to_vec();
    let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    else {
        return false;
    };
    if record.dispatching || !record.initialized {
        return false;
    }
    record.target = Some(target);
    record.current_target = None;
    record.event_phase = NONE;
    record.cancel_bubble = false;
    record.immediate_stopped = false;
    record.dispatching = true;
    record.in_passive_listener = false;
    record.path = path;
    true
}

pub(crate) fn set_current_target(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    target: v8::Local<'_, v8::Object>,
    phase: i32,
    exposed_target: v8::Local<'_, v8::Object>,
) {
    let target = v8::Global::new(scope, target);
    let exposed_target = v8::Global::new(scope, exposed_target);
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.current_target = Some(target);
        record.target = Some(exposed_target);
        record.event_phase = phase;
    }
}

pub(crate) fn set_target(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    target: v8::Local<'_, v8::Object>,
) {
    let target = v8::Global::new(scope, target);
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.target = Some(target);
    }
}

pub(crate) fn set_bubbles(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    bubbles: bool,
) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.bubbles = bubbles;
    }
}

pub(crate) fn set_passive_listener(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    passive: bool,
) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.in_passive_listener = passive;
    }
}

pub(crate) fn propagation_stopped(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, event).is_some_and(|record| record.cancel_bubble)
}

pub(crate) fn immediate_propagation_stopped(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, event).is_some_and(|record| record.immediate_stopped)
}

pub(crate) fn bubbles(scope: &v8::PinScope<'_, '_>, event: v8::Local<'_, v8::Object>) -> bool {
    record(scope, event).is_some_and(|record| record.bubbles)
}

pub(crate) fn composed(scope: &v8::PinScope<'_, '_>, event: v8::Local<'_, v8::Object>) -> bool {
    record(scope, event).is_some_and(|record| record.composed)
}

pub(crate) fn mark_uninitialized(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.initialized = false;
    }
}

pub(crate) fn default_prevented(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    scope
        .get_slot::<EventStore>()?
        .records
        .get(&event.get_identity_hash().get())
        .map(|record| record.default_prevented)
}

pub(crate) fn cancel(scope: &mut v8::PinScope<'_, '_>, event: v8::Local<'_, v8::Object>) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        if record.cancelable {
            record.default_prevented = true;
        }
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<EventRecord> {
    scope
        .get_slot::<EventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut EventRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn event_init(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> (bool, bool, bool) {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return (false, false, false);
    };
    (
        boolean_property(scope, object, "bubbles"),
        boolean_property(scope, object, "cancelable"),
        boolean_property(scope, object, "composed"),
    )
}

pub(crate) fn boolean_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    object
        .get(scope, key.into())
        .is_some_and(|value| !value.is_undefined() && value.boolean_value(scope))
}

pub(crate) fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: f64,
) -> f64 {
    let Some(key) = v8::String::new(scope, name) else {
        return default;
    };
    let Some(value) = object.get(scope, key.into()) else {
        return default;
    };
    if value.is_undefined() {
        default
    } else {
        value.number_value(scope).unwrap_or(f64::NAN)
    }
}

pub(crate) fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.event_type) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_target(
    scope: &mut v8::PinScope<'_, '_>,
    target: Option<v8::Global<v8::Object>>,
    result: &mut v8::ReturnValue<'_>,
) {
    if let Some(target) = target {
        result.set(v8::Local::new(scope, &target).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_target(scope, record.target, &mut result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_current_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_target(scope, record.current_target, &mut result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_event_phase(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.event_phase).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&EventRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_bubbles(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |record| record.bubbles);
}
pub(crate) fn get_cancelable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |record| record.cancelable);
}
pub(crate) fn get_default_prevented(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |record| record.default_prevented);
}
pub(crate) fn get_composed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |record| record.composed);
}
pub(crate) fn get_is_trusted(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |record| record.is_trusted);
}

pub(crate) fn set_trusted(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    trusted: bool,
) {
    if let Some(record) = scope
        .get_slot_mut::<EventStore>()
        .and_then(|store| store.records.get_mut(&event.get_identity_hash().get()))
    {
        record.is_trusted = trusted;
    }
}
pub(crate) fn get_cancel_bubble(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |record| record.cancel_bubble);
}

pub(crate) fn get_time_stamp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.time_stamp).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_src_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_target(scope, record.target, &mut result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_bool(scope, arguments, result, |record| !record.default_prevented);
}

pub(crate) fn set_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !arguments.get(0).boolean_value(scope) {
        update(scope, arguments.this(), |record| {
            if record.cancelable && !record.in_passive_listener {
                record.default_prevented = true;
            }
        });
    }
}

pub(crate) fn set_cancel_bubble(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if value {
        update(scope, arguments.this(), |record| {
            record.cancel_bubble = true
        });
    } else if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn composed_path(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let path = if record.dispatching {
        let current_index = record.current_target.as_ref().and_then(|current| {
            let current = v8::Local::new(scope, current);
            let current_id = current.get_identity_hash().get();
            record.path.iter().position(|entry| {
                v8::Local::new(scope, entry).get_identity_hash().get() == current_id
            })
        });
        let mut visible_start = 0;
        if let Some(current_index) = current_index {
            for (index, entry) in record.path.iter().enumerate().take(current_index) {
                let entry = v8::Local::new(scope, entry);
                if super::shadow_root::is_closed(scope, entry) {
                    visible_start = index + 1;
                }
            }
        }
        record.path.into_iter().skip(visible_start).collect()
    } else {
        Vec::new()
    };
    let array = v8::Array::new(scope, path.len() as i32);
    for (index, target) in path.iter().enumerate() {
        let target = v8::Local::new(scope, target);
        let _ = array.set_index(scope, index as u32, target.into());
    }
    result.set(array.into());
}

pub(crate) fn init_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    if record(scope, arguments.this()).is_some_and(|record| record.dispatching) {
        return;
    }
    update(scope, arguments.this(), |record| {
        record.event_type = event_type;
        record.bubbles = bubbles;
        record.cancelable = cancelable;
        record.composed = false;
        record.default_prevented = false;
        record.target = None;
        record.current_target = None;
        record.event_phase = NONE;
        record.cancel_bubble = false;
        record.immediate_stopped = false;
        record.initialized = true;
        record.path.clear();
    });
}

pub(crate) fn prevent_default(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        if record.cancelable && !record.in_passive_listener {
            record.default_prevented = true;
        }
    });
}

pub(crate) fn stop_immediate_propagation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.immediate_stopped = true;
        record.cancel_bubble = true;
    });
}

pub(crate) fn stop_propagation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.cancel_bubble = true
    });
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<EventStore>() {
        store.constructors.remove(&realm_id);
    }
}
