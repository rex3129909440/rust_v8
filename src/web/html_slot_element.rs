use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlSlotElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SlotRecord>,
}
#[derive(Clone, Default)]
pub(crate) struct SlotRecord {
    pub(crate) assigned: Vec<v8::Global<v8::Object>>,
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
    scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .ok_or_else(|| "HTMLSlotElement state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), SlotRecord::default());
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
pub(crate) fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        super::element::set_attribute_value(scope, a.this(), "name".to_owned(), v);
        dispatch_slotchange(scope, a.this());
    }
}
pub(crate) fn assign(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut assigned = Vec::new();
    for i in 0..a.length() {
        let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(i)) else {
            crate::webidl::throw_type_error(scope, "Assigned values must be Nodes");
            return;
        };
        if super::node::record(scope, node).is_none() {
            crate::webidl::throw_type_error(scope, "Assigned values must be Nodes");
            return;
        }
        assigned.push(v8::Global::new(scope, node));
    }
    if let Some(x) = scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.assigned = assigned;
        dispatch_slotchange(scope, a.this());
    }
}
pub(crate) fn current_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    slot: v8::Local<'s, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    let r = record(scope, slot)?;
    let assigned = if !r.assigned.is_empty() {
        r.assigned
            .iter()
            .map(|v| v8::Local::new(scope, v))
            .collect()
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
    super::node::children(scope, host)
        .into_iter()
        .filter(|node| slottable_name(scope, *node) == Some(wanted.clone()))
        .collect()
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
        if slot_record
            .assigned
            .iter()
            .any(|assigned| v8::Local::new(scope, assigned).strict_equals(node.into()))
        {
            return Some(slot);
        }
        if !manual
            && slot_record.assigned.is_empty()
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

pub(crate) fn dispatch_slotchange(
    scope: &mut v8::PinScope<'_, '_>,
    slot: v8::Local<'_, v8::Object>,
) {
    if let Ok(event) = super::event::create(scope, "slotchange") {
        super::event::set_bubbles(scope, event, true);
        let _ = super::event_target::dispatch(scope, slot, event);
    }
}

pub(crate) fn notify_assignment_change(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) {
    let mut roots = Vec::new();
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
        for slot in super::dom_selector::descendants(scope, root) {
            if record(scope, slot).is_some() {
                dispatch_slotchange(scope, slot);
            }
        }
    }
}

pub(crate) fn flatten_requested(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return false;
    };
    let Some(key) = v8::String::new(scope, "flatten") else {
        return false;
    };
    options
        .get(scope, key.into())
        .is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn selected_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    slot: v8::Local<'s, v8::Object>,
    options: v8::Local<'_, v8::Value>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    if flatten_requested(scope, options) {
        flattened_nodes(scope, slot)
    } else {
        current_nodes(scope, slot)
    }
}

pub(crate) fn assigned_nodes(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(nodes) = selected_nodes(scope, a.this(), a.get(0)) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let arr = v8::Array::new(scope, nodes.len() as i32);
    for (i, n) in nodes.iter().enumerate() {
        let _ = arr.set_index(scope, i as u32, (*n).into());
    }
    r.set(arr.into())
}
pub(crate) fn assigned_elements(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(nodes) = selected_nodes(scope, a.this(), a.get(0)) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let elements = nodes
        .into_iter()
        .filter(|n| super::element::record(scope, *n).is_some())
        .collect::<Vec<_>>();
    let arr = v8::Array::new(scope, elements.len() as i32);
    for (i, n) in elements.iter().enumerate() {
        let _ = arr.set_index(scope, i as u32, (*n).into());
    }
    r.set(arr.into())
}
