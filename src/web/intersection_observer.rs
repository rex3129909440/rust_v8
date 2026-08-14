use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IntersectionObserverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ObserverRecord>,
}

#[derive(Clone)]
struct ObserverRecord {
    callback: v8::Global<v8::Function>,
    observer: v8::Global<v8::Object>,
    root: Option<v8::Global<v8::Object>>,
    root_margin: String,
    scroll_margin: String,
    thresholds: Vec<f64>,
    delay: u32,
    track_visibility: bool,
    targets: Vec<ObservedTarget>,
    pending: Vec<v8::Global<v8::Object>>,
    delivery_scheduled: bool,
}

#[derive(Clone)]
struct ObservedTarget {
    identity: i32,
    object: v8::Global<v8::Object>,
    last_state: Option<(bool, u64)>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IntersectionObserverStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IntersectionObserver", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<IntersectionObserverStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IntersectionObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "root", get_root)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rootMargin", get_root_margin)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "scrollMargin", get_scroll_margin)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "thresholds", get_thresholds)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "delay", get_delay)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "trackVisibility",
        get_track_visibility,
    )?;
    crate::webidl::define_method(scope, prototype, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(scope, prototype, "observe", 1, observe)?;
    crate::webidl::define_method(scope, prototype, "takeRecords", 0, take_records)?;
    crate::webidl::define_method(scope, prototype, "unobserve", 1, unobserve)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IntersectionObserverStore>()
        .ok_or_else(|| "IntersectionObserver state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'IntersectionObserver': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'IntersectionObserver': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let root = options
        .and_then(|options| value_property(scope, options, "root"))
        .filter(|value| !value.is_null() && !value.is_undefined())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
    let root_margin = options
        .and_then(|options| string_property(scope, options, "rootMargin"))
        .unwrap_or_else(|| "0px".to_owned());
    let scroll_margin = options
        .and_then(|options| string_property(scope, options, "scrollMargin"))
        .unwrap_or_else(|| "0px".to_owned());
    let thresholds = match options
        .and_then(|options| value_property(scope, options, "threshold"))
        .map(|value| thresholds_from_value(scope, value))
        .unwrap_or_else(|| Ok(vec![0.0]))
    {
        Ok(thresholds) => thresholds,
        Err(message) => {
            let message = v8::String::new(scope, &message).unwrap();
            scope.throw_exception(v8::Exception::range_error(scope, message));
            return;
        }
    };
    let delay = options
        .and_then(|options| number_property(scope, options, "delay"))
        .unwrap_or(0.0)
        .max(0.0) as u32;
    let track_visibility = options
        .is_some_and(|options| super::event::boolean_property(scope, options, "trackVisibility"));
    let callback = v8::Global::new(scope, callback);
    let observer = v8::Global::new(scope, arguments.this());
    let root = root.map(|root| v8::Global::new(scope, root));
    scope
        .get_slot_mut::<IntersectionObserverStore>()
        .expect("IntersectionObserver state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ObserverRecord {
                callback,
                observer,
                root,
                root_margin: expand_margin(&root_margin),
                scroll_margin: expand_margin(&scroll_margin),
                thresholds,
                delay,
                track_visibility,
                targets: Vec::new(),
                pending: Vec::new(),
                delivery_scheduled: false,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ObserverRecord> {
    scope
        .get_slot::<IntersectionObserverStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_root(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(root) = record.root {
        result.set(v8::Local::new(scope, &root).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ObserverRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_root_margin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.root_margin);
}
fn get_scroll_margin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.scroll_margin);
}

fn get_thresholds(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.thresholds.len() as i32);
    for (index, threshold) in record.thresholds.iter().enumerate() {
        let value = v8::Number::new(scope, *threshold);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    result.set(array.into());
}

fn get_delay(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.delay).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_track_visibility(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.track_visibility).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn disconnect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.targets.clear();
        record.pending.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn observe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let Ok(target) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "target is not an Element");
        return;
    };
    if super::element::record(scope, target).is_none() {
        crate::webidl::throw_type_error(scope, "target is not an Element");
        return;
    }
    let id = target.get_identity_hash().get();
    let observed_target = ObservedTarget {
        identity: id,
        object: v8::Global::new(scope, target),
        last_state: None,
    };
    let observer_id = arguments.this().get_identity_hash().get();
    let Some(mut current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !current.targets.iter().any(|value| value.identity == id) {
        current.targets.push(observed_target);
    }
    if let Some(stored) = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        stored.targets = current.targets;
    }
    queue_delivery(scope, observer_id);
}

fn take_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let pending = std::mem::take(&mut record.pending);
    let array = v8::Array::new(scope, pending.len() as i32);
    for (index, entry) in pending.into_iter().enumerate() {
        let entry = v8::Local::new(scope, &entry);
        let _ = array.set_index(scope, index as u32, entry.into());
    }
    result.set(array.into());
}

fn unobserve(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(target) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "target is not an Element");
        return;
    };
    let target_id = target.get_identity_hash().get();
    let Some(mut current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    current.targets.retain(|value| value.identity != target_id);
    if let Some(stored) = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        stored.targets = current.targets;
    }
}

pub(crate) fn notify_target_change(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) {
    let target_id = target.get_identity_hash().get();
    let observer_ids = scope
        .get_slot::<IntersectionObserverStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter_map(|(observer_id, record)| {
                    record
                        .targets
                        .iter()
                        .any(|observed| observed.identity == target_id)
                        .then_some(*observer_id)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for observer_id in observer_ids {
        queue_delivery(scope, observer_id);
    }
}

fn queue_delivery(scope: &mut v8::PinScope<'_, '_>, observer_id: i32) {
    let should_schedule = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
        .is_some_and(|record| {
            if record.delivery_scheduled {
                false
            } else {
                record.delivery_scheduled = true;
                true
            }
        });
    if !should_schedule {
        return;
    }
    let data = v8::Integer::new(scope, observer_id);
    if let Some(function) = v8::Function::builder(deliver)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(function);
    }
}

fn deliver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(observer_id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some(record) = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
        .map(|record| {
            record.delivery_scheduled = false;
            record.clone()
        })
    else {
        return;
    };

    let root_rect = record
        .root
        .as_ref()
        .map(|root| {
            let root = v8::Local::new(scope, root);
            super::element_layout::compute(scope, root).rect()
        })
        .unwrap_or(super::dom_rect_read_only::RectRecord {
            x: 0.0,
            y: 0.0,
            width: super::window_view_state::inner_width(scope),
            height: super::window_view_state::inner_height(scope),
        });
    let root_object = super::dom_rect_read_only::create(scope, root_rect).ok();
    let mut entries = Vec::new();
    let mut delivered_states = Vec::new();
    for target in &record.targets {
        let target_object = v8::Local::new(scope, &target.object);
        let target_rect = super::element_layout::compute(scope, target_object).rect();
        let intersection = intersection_rect(target_rect, root_rect);
        let target_area = (target_rect.width * target_rect.height).max(0.0);
        let intersection_area = (intersection.width * intersection.height).max(0.0);
        let is_intersecting = target_rect.width > 0.0
            && target_rect.height > 0.0
            && intersection.width > 0.0
            && intersection.height > 0.0;
        let ratio = if target_area > 0.0 {
            (intersection_area / target_area).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let state = (is_intersecting, ratio.to_bits());
        if target.last_state == Some(state) {
            continue;
        }
        let Ok(bounds) = super::dom_rect_read_only::create(scope, target_rect) else {
            continue;
        };
        let Ok(intersection_bounds) = super::dom_rect_read_only::create(scope, intersection) else {
            continue;
        };
        let entry = match super::intersection_observer_entry::create(
            scope,
            super::performance::now_for_current_realm(scope).unwrap_or(0.0),
            root_object,
            bounds,
            intersection_bounds,
            is_intersecting,
            record.track_visibility && is_intersecting,
            ratio,
            target_object,
        ) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        entries.push(v8::Global::new(scope, entry));
        delivered_states.push((target.identity, state));
    }

    if let Some(stored) = scope
        .get_slot_mut::<IntersectionObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
    {
        for (target_id, state) in delivered_states {
            if let Some(target) = stored
                .targets
                .iter_mut()
                .find(|target| target.identity == target_id)
            {
                target.last_state = Some(state);
            }
        }
    }
    if entries.is_empty() {
        return;
    }
    let values = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let _ = values.set_index(scope, index as u32, v8::Local::new(scope, entry).into());
    }
    let callback = v8::Local::new(scope, &record.callback);
    let observer = v8::Local::new(scope, &record.observer);
    let _ = callback.call(
        scope,
        v8::undefined(scope).into(),
        &[values.into(), observer.into()],
    );
}

fn intersection_rect(
    target: super::dom_rect_read_only::RectRecord,
    root: super::dom_rect_read_only::RectRecord,
) -> super::dom_rect_read_only::RectRecord {
    let left = target.x.max(root.x);
    let top = target.y.max(root.y);
    let right = (target.x + target.width).min(root.x + root.width);
    let bottom = (target.y + target.height).min(root.y + root.height);
    if right <= left || bottom <= top {
        return super::dom_rect_read_only::RectRecord {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
    }
    super::dom_rect_read_only::RectRecord {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

fn thresholds_from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<f64>, String> {
    let mut values = Vec::new();
    if let Ok(array) = v8::Local::<v8::Array>::try_from(value) {
        for index in 0..array.length() {
            let threshold = array
                .get_index(scope, index)
                .and_then(|value| value.number_value(scope))
                .unwrap_or(f64::NAN);
            values.push(threshold);
        }
    } else {
        values.push(value.number_value(scope).unwrap_or(f64::NAN));
    }
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
    {
        return Err(
            "Failed to construct 'IntersectionObserver': Threshold values must be numbers between 0 and 1"
                .to_owned(),
        );
    }
    values.sort_by(|a, b| a.total_cmp(b));
    values.dedup();
    if values.is_empty() {
        values.push(0.0);
    }
    Ok(values)
}

fn expand_margin(value: &str) -> String {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        [] => "0px 0px 0px 0px".to_owned(),
        [a] => format!("{a} {a} {a} {a}"),
        [a, b] => format!("{a} {b} {a} {b}"),
        [a, b, c] => format!("{a} {b} {c} {b}"),
        [a, b, c, d, ..] => format!("{a} {b} {c} {d}"),
    }
}

fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then_some(value)
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    value_property(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}

fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    value_property(scope, object, name).and_then(|value| value.number_value(scope))
}
