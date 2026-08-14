use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlSlotElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SlotRecord>,
}
#[derive(Clone, Default)]
pub(crate) struct SlotRecord {
    pub(crate) object: Option<v8::Global<v8::Object>>,
    pub(crate) assigned: Vec<v8::Global<v8::Object>>,
    pub(crate) last_effective_assignment: Vec<i32>,
    pub(crate) slotchange_pending: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlSlotElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLSlotElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlSlotElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLSlotElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_slot_element_name_property::define(scope, p)?;
    super::html_slot_element_assign::define(scope, p)?;
    super::html_slot_element_assigned_elements::define(scope, p)?;
    super::html_slot_element_assigned_nodes::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .ok_or_else(|| "HTMLSlotElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create HTMLSlotElement".to_owned());
    }
    super::html_element::attach(scope, o, "SLOT");
    let object = v8::Global::new(scope, o);
    scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .ok_or_else(|| "HTMLSlotElement state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            SlotRecord {
                object: Some(object),
                ..SlotRecord::default()
            },
        );
    Ok(o)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<SlotRecord> {
    scope
        .get_slot::<HtmlSlotElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        let name = super::element::attribute_value(scope, a.this(), "name").unwrap_or_default();
        if let Some(v) = v8::String::new(scope, &name) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_name_impl(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(value) = crate::webidl::dom_string_with_context(
        scope,
        a.get(0),
        "Failed to set the 'name' property on 'HTMLSlotElement'",
    ) else {
        return;
    };
    super::element::set_attribute_value(scope, a.this(), "name".to_owned(), value);
}
pub(crate) fn assign_impl(scope: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut assigned = Vec::new();
    let mut seen = Vec::new();
    for i in 0..a.length() {
        let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(i)) else {
            throw_assign_type_error(scope);
            return;
        };
        if !is_slottable(scope, node) {
            throw_assign_type_error(scope);
            return;
        }
        let identity = node.get_identity_hash().get();
        if !seen.contains(&identity) {
            seen.push(identity);
            assigned.push(v8::Global::new(scope, node));
        }
    }
    let Some(root) = containing_shadow_root(scope, a.this()) else {
        return;
    };
    if !super::shadow_root::uses_manual_slot_assignment(scope, root) {
        return;
    }
    let selected = seen;
    let slot_identity = a.this().get_identity_hash().get();
    let slot_ids = super::dom_selector::descendants(scope, root)
        .into_iter()
        .filter(|candidate| record(scope, *candidate).is_some())
        .map(|candidate| candidate.get_identity_hash().get())
        .collect::<Vec<_>>();
    if let Some(x) = scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .and_then(|s| s.records.get_mut(&slot_identity))
    {
        x.assigned = assigned;
    }
    for candidate_id in slot_ids {
        if candidate_id == slot_identity {
            continue;
        }
        let retained = scope
            .get_slot::<HtmlSlotElementStore>()
            .and_then(|store| store.records.get(&candidate_id))
            .map(|candidate| {
                candidate
                    .assigned
                    .iter()
                    .filter(|node| {
                        let node = v8::Local::new(scope, *node);
                        !selected.contains(&node.get_identity_hash().get())
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(candidate) = scope
            .get_slot_mut::<HtmlSlotElementStore>()
            .and_then(|store| store.records.get_mut(&candidate_id))
        {
            candidate.assigned = retained;
        }
    }
    queue_assignment_changes_for_root(scope, root);
}
pub(crate) fn current_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    slot: v8::Local<'s, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    let root = containing_shadow_root(scope, slot);
    let assigned =
        if root.is_some_and(|root| super::shadow_root::uses_manual_slot_assignment(scope, root)) {
            manually_assigned_nodes(scope, slot, root.unwrap())
        } else {
            automatically_assigned_nodes(scope, slot)
        };
    Some(assigned)
}

pub(crate) fn automatically_assigned_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    slot: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let Some(root) = containing_shadow_root(scope, slot) else {
        return Vec::new();
    };
    if super::shadow_root::uses_manual_slot_assignment(scope, root) {
        return Vec::new();
    }
    let Some(host) = super::shadow_root::host(scope, root) else {
        return Vec::new();
    };
    let wanted = super::element::attribute_value(scope, slot, "name").unwrap_or_default();
    let first_matching_slot = super::dom_selector::descendants(scope, root)
        .into_iter()
        .find(|candidate| {
            record(scope, *candidate).is_some()
                && super::element::attribute_value(scope, *candidate, "name").unwrap_or_default()
                    == wanted
        });
    if !first_matching_slot.is_some_and(|candidate| candidate.strict_equals(slot.into())) {
        return Vec::new();
    }
    super::node::children(scope, host)
        .into_iter()
        .filter(|node| slottable_name(scope, *node) == Some(wanted.clone()))
        .collect()
}

fn manually_assigned_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    slot: v8::Local<'s, v8::Object>,
    root: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let Some(host) = super::shadow_root::host(scope, root) else {
        return Vec::new();
    };
    record(scope, slot)
        .map(|record| {
            record
                .assigned
                .iter()
                .map(|node| v8::Local::new(scope, node))
                .filter(|node| {
                    super::node::parent(scope, *node)
                        .is_some_and(|parent| parent.strict_equals(host.into()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn is_slottable(scope: &v8::PinScope<'_, '_>, node: v8::Local<'_, v8::Object>) -> bool {
    super::node::record(scope, node).is_some_and(|record| matches!(record.node_type, 1 | 3))
}

fn throw_assign_type_error(scope: &mut v8::PinScope<'_, '_>) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'assign' on 'HTMLSlotElement': The provided value is not of type '(Element or Text)'.",
    );
}

pub(crate) fn containing_shadow_root<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut cursor = node;
    while let Some(parent) = super::node::parent(scope, cursor) {
        if super::shadow_root::host(scope, parent).is_some() {
            return Some(parent);
        }
        cursor = parent;
    }
    None
}

pub(crate) fn slottable_name(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let node_type = super::node::record(scope, node)?.node_type;
    match node_type {
        1 => Some(super::element::attribute_value(scope, node, "slot").unwrap_or_default()),
        3 => Some(String::new()),
        _ => None,
    }
}

pub(crate) fn assigned_slot<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = super::node::parent(scope, node)?;
    let shadow = super::element::record(scope, parent)?.shadow_root?;
    let shadow = v8::Local::new(scope, &shadow);
    let wanted = slottable_name(scope, node)?;
    let manual = super::shadow_root::uses_manual_slot_assignment(scope, shadow);
    for slot in super::dom_selector::descendants(scope, shadow) {
        let Some(slot_record) = record(scope, slot) else {
            continue;
        };
        if manual
            && slot_record
                .assigned
                .iter()
                .any(|assigned| v8::Local::new(scope, assigned).strict_equals(node.into()))
        {
            return Some(slot);
        }
        if !manual
            && super::element::attribute_value(scope, slot, "name").unwrap_or_default() == wanted
        {
            return Some(slot);
        }
    }
    None
}

pub(crate) fn flattened_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    slot: v8::Local<'s, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    if containing_shadow_root(scope, slot).is_none() {
        return Some(Vec::new());
    }
    let mut output = Vec::new();
    let mut nodes = current_nodes(scope, slot)?;
    if nodes.is_empty() {
        nodes = super::node::children(scope, slot);
    }
    for node in nodes {
        if record(scope, node).is_some() {
            if let Some(nested) = flattened_nodes(scope, node) {
                output.extend(nested);
            }
        } else {
            output.push(node);
        }
    }
    Some(output)
}

fn dispatch_slotchange(scope: &mut v8::PinScope<'_, '_>, slot: v8::Local<'_, v8::Object>) {
    if let Ok(event) = super::event::create(scope, "slotchange") {
        super::event::set_bubbles(scope, event, true);
        super::event::set_trusted(scope, event, true);
        let _ = super::event_target::dispatch(scope, slot, event);
    }
}

fn deliver_slotchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(identity) = arguments.data().int32_value(scope) else {
        return;
    };
    let pending = scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
        .is_some_and(|record| {
            let pending = record.slotchange_pending;
            record.slotchange_pending = false;
            pending
        });
    if !pending {
        return;
    }
    let slot = scope
        .get_slot::<HtmlSlotElementStore>()
        .and_then(|store| store.records.get(&identity))
        .and_then(|record| record.object.as_ref())
        .map(|slot| v8::Local::new(scope, slot));
    let Some(slot) = slot else {
        return;
    };
    dispatch_slotchange(scope, slot);
}

fn queue_slotchange(scope: &mut v8::PinScope<'_, '_>, slot: v8::Local<'_, v8::Object>) {
    let identity = slot.get_identity_hash().get();
    let should_queue = scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
        .is_some_and(|record| {
            if record.slotchange_pending {
                false
            } else {
                record.slotchange_pending = true;
                true
            }
        });
    if should_queue {
        let data = v8::Integer::new(scope, identity);
        if let Some(callback) = v8::Function::builder(deliver_slotchange)
            .data(data.into())
            .length(0)
            .constructor_behavior(v8::ConstructorBehavior::Throw)
            .build(scope)
        {
            scope.enqueue_microtask(callback);
        }
    }
}

fn queue_assignment_changes_for_root(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    let slots = super::dom_selector::descendants(scope, root)
        .into_iter()
        .filter(|slot| record(scope, *slot).is_some())
        .collect::<Vec<_>>();
    for slot in slots {
        let effective = current_nodes(scope, slot)
            .unwrap_or_default()
            .iter()
            .map(|node| node.get_identity_hash().get())
            .collect::<Vec<_>>();
        let changed = scope
            .get_slot_mut::<HtmlSlotElementStore>()
            .and_then(|store| store.records.get_mut(&slot.get_identity_hash().get()))
            .is_some_and(|record| {
                if record.last_effective_assignment == effective {
                    false
                } else {
                    record.last_effective_assignment = effective;
                    true
                }
            });
        if changed {
            queue_slotchange(scope, slot);
        }
    }
}

pub(crate) fn notify_assignment_change(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) {
    let mut roots = Vec::new();
    if super::shadow_root::record(scope, node).is_some() {
        roots.push(v8::Global::new(scope, node));
    }
    if let Some(root) = super::element::record(scope, node).and_then(|record| record.shadow_root) {
        roots.push(root);
    }
    if let Some(parent) = super::node::parent(scope, node) {
        if let Some(root) =
            super::element::record(scope, parent).and_then(|record| record.shadow_root)
        {
            roots.push(root);
        }
    }
    if let Some(root) = containing_shadow_root(scope, node) {
        roots.push(v8::Global::new(scope, root));
    }
    let mut seen = Vec::new();
    for root in roots {
        let root = v8::Local::new(scope, &root);
        let identity = root.get_identity_hash().get();
        if seen.contains(&identity) {
            continue;
        }
        seen.push(identity);
        queue_assignment_changes_for_root(scope, root);
    }
}

pub(crate) fn flatten_requested(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<bool> {
    if value.is_undefined() || value.is_null() {
        return Some(false);
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'assignedNodes' on 'HTMLSlotElement': The provided value is not of type 'AssignedNodesOptions'.",
        );
        return None;
    };
    let Some(key) = v8::String::new(scope, "flatten") else {
        return Some(false);
    };
    options
        .get(scope, key.into())
        .map(|value| value.boolean_value(scope))
}
