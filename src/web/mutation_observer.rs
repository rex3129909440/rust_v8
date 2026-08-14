use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MutationObserverStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) observers: HashMap<i32, ObserverRecord>,
    pub(crate) next_registration_order: u64,
    pub(crate) suppressed_child_list_targets: std::collections::HashSet<i32>,
}

#[derive(Clone)]
pub(crate) struct ObserverRecord {
    pub(crate) callback: v8::Global<v8::Function>,
    pub(crate) observer: v8::Global<v8::Object>,
    pub(crate) observed_targets: Vec<ObservedTarget>,
    pub(crate) pending: Vec<v8::Global<v8::Object>>,
    pub(crate) microtask_scheduled: bool,
    pub(crate) registration_order: u64,
    pub(crate) transient_observed_targets: Vec<TransientObservedTarget>,
}

#[derive(Clone)]
pub(crate) struct ObservedTarget {
    pub(crate) target: v8::Global<v8::Object>,
    pub(crate) child_list: bool,
    pub(crate) attributes: bool,
    pub(crate) character_data: bool,
    pub(crate) subtree: bool,
    pub(crate) attribute_old_value: bool,
    pub(crate) character_data_old_value: bool,
    pub(crate) attribute_filter: Option<Vec<String>>,
}

#[derive(Clone)]
pub(crate) struct TransientObservedTarget {
    pub(crate) root_id: i32,
    pub(crate) root: v8::Global<v8::Object>,
    pub(crate) observed: ObservedTarget,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MutationObserverStore::default());
}

pub(crate) fn install_standard_name(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MutationObserver", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MutationObserverStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MutationObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::mutation_observer_disconnect::define(scope, prototype)?;
    super::mutation_observer_observe::define(scope, prototype)?;
    super::mutation_observer_take_records::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MutationObserverStore>()
        .ok_or_else(|| "MutationObserver state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MutationObserver': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MutationObserver': 1 argument required",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MutationObserver': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let observer = arguments.this();
    let callback = v8::Global::new(scope, callback);
    let observer_global = v8::Global::new(scope, observer);
    let store = scope
        .get_slot_mut::<MutationObserverStore>()
        .expect("MutationObserver state");
    let registration_order = store.next_registration_order;
    store.next_registration_order = store.next_registration_order.saturating_add(1);
    store.observers.insert(
        observer.get_identity_hash().get(),
        ObserverRecord {
            callback,
            observer: observer_global,
            observed_targets: Vec::new(),
            pending: Vec::new(),
            microtask_scheduled: false,
            registration_order,
            transient_observed_targets: Vec::new(),
        },
    );
    result.set(observer.into());
}

pub(crate) fn enqueue_attribute_change(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    attribute_name: String,
    attribute_namespace: Option<String>,
    old_value: Option<String>,
) {
    let observers = observer_ids_for(scope, target, |observed| {
        observed.attributes
            && observed.attribute_filter.as_ref().is_none_or(|filter| {
                let local_name = attribute_name.rsplit(':').next().unwrap_or(&attribute_name);
                filter.iter().any(|candidate| candidate == local_name)
            })
    });
    for (observer_id, observed) in observers {
        let reported_old_value = observed
            .attribute_old_value
            .then(|| old_value.clone())
            .flatten();
        let Ok(record) = super::mutation_record::create_attribute(
            scope,
            target,
            attribute_name.clone(),
            attribute_namespace.clone(),
            reported_old_value,
        ) else {
            continue;
        };
        queue_record(scope, observer_id, record);
    }
}

pub(crate) fn enqueue_child_list(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    added_nodes: Vec<v8::Local<'_, v8::Object>>,
    removed_nodes: Vec<v8::Local<'_, v8::Object>>,
    previous_sibling: Option<v8::Local<'_, v8::Object>>,
    next_sibling: Option<v8::Local<'_, v8::Object>>,
) {
    if scope
        .get_slot::<MutationObserverStore>()
        .is_some_and(|store| {
            store
                .suppressed_child_list_targets
                .contains(&target.get_identity_hash().get())
        })
    {
        return;
    }
    let observers = observer_ids_for(scope, target, |observed| observed.child_list);
    for (observer_id, observed) in observers {
        if observed.subtree && !removed_nodes.is_empty() {
            let roots = removed_nodes
                .iter()
                .map(|root| TransientObservedTarget {
                    root_id: root.get_identity_hash().get(),
                    root: v8::Global::new(scope, *root),
                    observed: observed.clone(),
                })
                .collect::<Vec<_>>();
            if let Some(observer) = scope
                .get_slot_mut::<MutationObserverStore>()
                .and_then(|store| store.observers.get_mut(&observer_id))
            {
                for root in roots {
                    if !observer
                        .transient_observed_targets
                        .iter()
                        .any(|existing| existing.root_id == root.root_id)
                    {
                        observer.transient_observed_targets.push(root);
                    }
                }
            }
        }
        let Ok(record) = super::mutation_record::create(
            scope,
            "childList",
            target,
            added_nodes.clone(),
            removed_nodes.clone(),
            previous_sibling,
            next_sibling,
            None,
            None,
            None,
        ) else {
            continue;
        };
        queue_record(scope, observer_id, record);
    }
}

pub(crate) fn enqueue_character_data_change(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    old_value: Option<String>,
) {
    let observers = observer_ids_for(scope, target, |observed| observed.character_data);
    for (observer_id, observed) in observers {
        let reported_old_value = observed
            .character_data_old_value
            .then(|| old_value.clone())
            .flatten();
        let Ok(record) = super::mutation_record::create(
            scope,
            "characterData",
            target,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None,
            reported_old_value,
        ) else {
            continue;
        };
        queue_record(scope, observer_id, record);
    }
}

pub(crate) fn observer_ids_for(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    enabled: impl Fn(&ObservedTarget) -> bool,
) -> Vec<(i32, ObservedTarget)> {
    let Some(store) = scope.get_slot::<MutationObserverStore>() else {
        return Vec::new();
    };
    let mut matches = Vec::new();
    for (observer_id, observer) in &store.observers {
        if let Some(observed) = observer
            .observed_targets
            .iter()
            .find(|observed| enabled(observed) && observed_matches(scope, observed, target))
        {
            matches.push((*observer_id, observed.clone()));
        } else if let Some(transient) = observer.transient_observed_targets.iter().find(|entry| {
            enabled(&entry.observed) && inclusive_descendant_of(scope, target, &entry.root)
        }) {
            matches.push((*observer_id, transient.observed.clone()));
        }
    }
    matches.sort_by_key(|(observer_id, _)| {
        store
            .observers
            .get(observer_id)
            .map(|observer| observer.registration_order)
            .unwrap_or(u64::MAX)
    });
    matches
}

fn inclusive_descendant_of(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    root: &v8::Global<v8::Object>,
) -> bool {
    let root = v8::Local::new(scope, root);
    let mut cursor = Some(target);
    while let Some(node) = cursor {
        if node.strict_equals(root.into()) {
            return true;
        }
        cursor = super::node::parent(scope, node);
    }
    false
}

pub(crate) fn observed_matches(
    scope: &v8::PinScope<'_, '_>,
    observed: &ObservedTarget,
    target: v8::Local<'_, v8::Object>,
) -> bool {
    let observed_target = v8::Local::new(scope, &observed.target);
    if observed_target.get_identity_hash().get() == target.get_identity_hash().get() {
        return true;
    }
    if !observed.subtree {
        return false;
    }
    let mut cursor = super::node::record(scope, target).and_then(|record| record.parent);
    while let Some(parent) = cursor {
        let parent = v8::Local::new(scope, &parent);
        if parent.get_identity_hash().get() == observed_target.get_identity_hash().get() {
            return true;
        }
        cursor = super::node::record(scope, parent).and_then(|record| record.parent);
    }
    false
}

pub(crate) fn queue_record(
    scope: &mut v8::PinScope<'_, '_>,
    observer_id: i32,
    record: v8::Local<'_, v8::Object>,
) {
    let record = v8::Global::new(scope, record);
    let should_schedule = scope
        .get_slot_mut::<MutationObserverStore>()
        .and_then(|store| store.observers.get_mut(&observer_id))
        .is_some_and(|observer| {
            observer.pending.push(record);
            if observer.microtask_scheduled {
                false
            } else {
                observer.microtask_scheduled = true;
                true
            }
        });
    if should_schedule {
        let data = v8::Integer::new(scope, observer_id);
        if let Some(function) = v8::Function::builder(deliver_records)
            .data(data.into())
            .length(0)
            .constructor_behavior(v8::ConstructorBehavior::Throw)
            .build(scope)
        {
            scope.enqueue_microtask(function);
        }
    }
}

pub(crate) fn deliver_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(observer_id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some((callback, observer, records)) = scope
        .get_slot_mut::<MutationObserverStore>()
        .and_then(|store| store.observers.get_mut(&observer_id))
        .and_then(|observer| {
            observer.microtask_scheduled = false;
            observer.transient_observed_targets.clear();
            if observer.pending.is_empty() {
                return None;
            }
            Some((
                observer.callback.clone(),
                observer.observer.clone(),
                std::mem::take(&mut observer.pending),
            ))
        })
    else {
        return;
    };
    let callback = v8::Local::new(scope, &callback);
    let observer = v8::Local::new(scope, &observer);
    let list = records_array(scope, &records);
    let _ = callback.call(scope, observer.into(), &[list.into(), observer.into()]);
}

pub(crate) fn suppress_child_list_for(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) {
    if let Some(store) = scope.get_slot_mut::<MutationObserverStore>() {
        store
            .suppressed_child_list_targets
            .insert(target.get_identity_hash().get());
    }
}

pub(crate) fn unsuppress_child_list_for(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) {
    if let Some(store) = scope.get_slot_mut::<MutationObserverStore>() {
        store
            .suppressed_child_list_targets
            .remove(&target.get_identity_hash().get());
    }
}

pub(crate) fn records_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    records: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, records.len() as i32);
    for (index, record) in records.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, record).into());
    }
    array
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn boolean_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    property(scope, object, name).is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn optional_boolean_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<bool> {
    property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .map(|value| value.boolean_value(scope))
}

pub(crate) fn string_sequence_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<Vec<String>> {
    let value = property(scope, object, name)?;
    if value.is_undefined() {
        return None;
    }
    let sequence = crate::webidl::sequence_values(scope, value).ok()?;
    Some(
        sequence
            .iter()
            .map(|value| {
                let value = v8::Local::new(scope, value);
                crate::webidl::value_to_string(scope, value)
            })
            .collect(),
    )
}
