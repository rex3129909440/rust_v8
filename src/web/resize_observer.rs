use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ResizeObserverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ObserverRecord>,
}

#[derive(Clone)]
struct ObserverRecord {
    callback: v8::Global<v8::Function>,
    observer: v8::Global<v8::Object>,
    targets: Vec<ObservedTarget>,
    delivery_scheduled: bool,
}

#[derive(Clone)]
struct ObservedTarget {
    identity: i32,
    object: v8::Global<v8::Object>,
    observed_box: ObservedBox,
    last_size: Option<(f64, f64)>,
}

#[derive(Clone, Copy)]
enum ObservedBox {
    Content,
    Border,
    DevicePixelContent,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ResizeObserverStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ResizeObserver", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ResizeObserverStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ResizeObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(scope, prototype, "observe", 1, observe)?;
    crate::webidl::define_method(scope, prototype, "unobserve", 1, unobserve)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ResizeObserverStore>()
        .ok_or_else(|| "ResizeObserver state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ResizeObserver': use the new operator",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ResizeObserver': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let record = ObserverRecord {
        callback: v8::Global::new(scope, callback),
        observer: v8::Global::new(scope, arguments.this()),
        targets: Vec::new(),
        delivery_scheduled: false,
    };
    scope
        .get_slot_mut::<ResizeObserverStore>()
        .expect("ResizeObserver state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn disconnect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<ResizeObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.targets.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn observe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<ResizeObserverStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(target) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "ResizeObserver target must be an Element");
        return;
    };
    if super::element::record(scope, target).is_none() {
        crate::webidl::throw_type_error(scope, "ResizeObserver target must be an Element");
        return;
    }
    let observed_box = match box_option(scope, arguments.get(1)).as_deref() {
        None | Some("content-box") => ObservedBox::Content,
        Some("border-box") => ObservedBox::Border,
        Some("device-pixel-content-box") => ObservedBox::DevicePixelContent,
        Some(_) => {
            crate::webidl::throw_type_error(scope, "ResizeObserver box option is invalid");
            return;
        }
    };
    let identity = target.get_identity_hash().get();
    let observed_target = ObservedTarget {
        identity,
        object: v8::Global::new(scope, target),
        observed_box,
        last_size: None,
    };
    let observer_id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<ResizeObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(existing) = record
        .targets
        .iter_mut()
        .find(|existing| existing.identity == identity)
    {
        *existing = observed_target;
    } else {
        record.targets.push(observed_target);
    }
    queue_delivery(scope, observer_id);
}

fn unobserve(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<ResizeObserverStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(target) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "ResizeObserver target must be an Element");
        return;
    };
    if super::element::record(scope, target).is_none() {
        crate::webidl::throw_type_error(scope, "ResizeObserver target must be an Element");
        return;
    }
    let identity = target.get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<ResizeObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record
        .targets
        .retain(|existing| existing.identity != identity);
}

pub(crate) fn notify(
    scope: &mut v8::PinScope<'_, '_>,
    observer: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let observer_id = observer.get_identity_hash().get();
    if scope
        .get_slot::<ResizeObserverStore>()
        .is_none_or(|store| !store.records.contains_key(&observer_id))
    {
        return Err("Illegal ResizeObserver".to_owned());
    }
    queue_delivery(scope, observer_id);
    Ok(())
}

pub(crate) fn notify_target_change(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) {
    let target_id = target.get_identity_hash().get();
    let observer_ids = scope
        .get_slot::<ResizeObserverStore>()
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
        .get_slot_mut::<ResizeObserverStore>()
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
        .get_slot_mut::<ResizeObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
        .map(|record| {
            record.delivery_scheduled = false;
            record.clone()
        })
    else {
        return;
    };

    let mut created = Vec::new();
    let mut delivered_sizes = Vec::new();
    for target in &record.targets {
        let target_object = v8::Local::new(scope, &target.object);
        let layout = super::element_layout::compute(scope, target_object);
        let device_scale = super::window_view_state::device_pixel_ratio(scope);
        let selected_size = match target.observed_box {
            ObservedBox::Content => (layout.content_width, layout.content_height),
            ObservedBox::Border => (layout.border_width(), layout.border_height()),
            ObservedBox::DevicePixelContent => (
                (layout.content_width * device_scale).round(),
                (layout.content_height * device_scale).round(),
            ),
        };
        if target.last_size == Some(selected_size) {
            continue;
        }
        let entry = match super::resize_observer_entry::create(
            scope,
            target_object,
            layout.content_width,
            layout.content_height,
            layout.border_width(),
            layout.border_height(),
            (layout.content_width * device_scale).round(),
            (layout.content_height * device_scale).round(),
        ) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        created.push(v8::Global::new(scope, entry));
        delivered_sizes.push((target.identity, selected_size));
    }

    if let Some(stored) = scope
        .get_slot_mut::<ResizeObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
    {
        for (target_id, size) in delivered_sizes {
            if let Some(target) = stored
                .targets
                .iter_mut()
                .find(|target| target.identity == target_id)
            {
                target.last_size = Some(size);
            }
        }
    }
    if created.is_empty() {
        return;
    }
    let entries = v8::Array::new(scope, created.len() as i32);
    for (index, entry) in created.iter().enumerate() {
        let _ = entries.set_index(scope, index as u32, v8::Local::new(scope, entry).into());
    }
    let callback = v8::Local::new(scope, &record.callback);
    let observer = v8::Local::new(scope, &record.observer);
    let _ = callback.call(
        scope,
        v8::undefined(scope).into(),
        &[entries.into(), observer.into()],
    );
}

fn box_option(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Option<String> {
    let options = v8::Local::<v8::Object>::try_from(value).ok()?;
    let key = v8::String::new(scope, "box")?;
    let value = options.get(scope, key.into())?;
    (!value.is_undefined()).then(|| crate::webidl::value_to_string(scope, value))
}
