use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NodeIteratorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NodeIteratorRecord>,
}

#[derive(Clone)]
pub(crate) struct NodeIteratorRecord {
    pub id: i32,
    pub root: v8::Global<v8::Object>,
    pub reference: v8::Global<v8::Object>,
    pub pointer_before_reference_node: bool,
    pub what_to_show: u32,
    pub filter: Option<v8::Global<v8::Object>>,
    pub active: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NodeIteratorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NodeIterator", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NodeIteratorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NodeIterator",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::node_iterator_root_property::define(scope, prototype)?;
    super::node_iterator_reference_node_property::define(scope, prototype)?;
    super::node_iterator_pointer_before_reference_node_property::define(scope, prototype)?;
    super::node_iterator_what_to_show_property::define(scope, prototype)?;
    super::node_iterator_filter_property::define(scope, prototype)?;
    super::node_iterator_detach::define(scope, prototype)?;
    super::node_iterator_next_node::define(scope, prototype)?;
    super::node_iterator_previous_node::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NodeIteratorStore>()
        .ok_or_else(|| "NodeIterator state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NodeIterator': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
    what_to_show: u32,
    filter: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if super::node::record(scope, root).is_none() {
        return Err("NodeIterator root must be a Node".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let iterator = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, iterator, prototype.into()) != Some(true) {
        return Err("cannot create NodeIterator".to_owned());
    }
    let root_global = v8::Global::new(scope, root);
    let reference = v8::Global::new(scope, root);
    let filter = filter.map(|filter| v8::Global::new(scope, filter));
    let id = iterator.get_identity_hash().get();
    scope
        .get_slot_mut::<NodeIteratorStore>()
        .ok_or_else(|| "NodeIterator state was not prepared".to_owned())?
        .records
        .insert(
            id,
            NodeIteratorRecord {
                id,
                root: root_global,
                reference,
                pointer_before_reference_node: true,
                what_to_show,
                filter,
                active: false,
            },
        );
    Ok(iterator)
}

pub(crate) fn collect_nodes<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
    output: &mut Vec<v8::Local<'s, v8::Object>>,
) {
    output.push(node);
    for child in super::node::children(scope, node) {
        collect_nodes(scope, child, output);
    }
}

pub(crate) fn following_node<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
    root: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    if let Some(child) = super::node::children(scope, node).into_iter().next() {
        return Some(child);
    }
    let mut current = node;
    loop {
        if current.strict_equals(root.into()) {
            return None;
        }
        let parent = super::node::parent(scope, current)?;
        let children = super::node::children(scope, parent);
        let index = children
            .iter()
            .position(|child| child.strict_equals(current.into()))?;
        if let Some(sibling) = children.get(index + 1) {
            return Some(*sibling);
        }
        current = parent;
    }
}

pub(crate) fn preceding_node<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
    root: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    if node.strict_equals(root.into()) {
        return None;
    }
    let parent = super::node::parent(scope, node)?;
    let children = super::node::children(scope, parent);
    let index = children
        .iter()
        .position(|child| child.strict_equals(node.into()))?;
    if let Some(previous) = index.checked_sub(1).and_then(|index| children.get(index)) {
        let mut candidate = *previous;
        loop {
            let descendants = super::node::children(scope, candidate);
            let Some(last) = descendants.last() else {
                return Some(candidate);
            };
            candidate = *last;
        }
    }
    Some(parent)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NodeIteratorRecord> {
    scope
        .get_slot::<NodeIteratorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn accepts(
    scope: &mut v8::PinScope<'_, '_>,
    record: &NodeIteratorRecord,
    node: v8::Local<'_, v8::Object>,
    operation: &str,
) -> Result<bool, ()> {
    let node_type = super::node::record(scope, node)
        .map(|record| record.node_type)
        .unwrap_or(0);
    let mask = if node_type > 0 && node_type <= 32 {
        1_u32 << (node_type as u32 - 1)
    } else {
        0
    };
    if record.what_to_show != u32::MAX && record.what_to_show & mask == 0 {
        return Ok(false);
    }
    let Some(filter) = &record.filter else {
        return Ok(true);
    };
    let filter = v8::Local::new(scope, filter);
    let (function, receiver) = if let Ok(function) = v8::Local::<v8::Function>::try_from(filter) {
        (function, v8::undefined(scope).into())
    } else {
        let Some(key) = v8::String::new(scope, "acceptNode") else {
            return Err(());
        };
        let Some(value) = filter.get(scope, key.into()) else {
            return Err(());
        };
        let Ok(function) = v8::Local::<v8::Function>::try_from(value) else {
            crate::webidl::throw_type_error(
                scope,
                &format!(
                    "Failed to execute '{operation}' on 'NodeIterator': Failed to execute 'acceptNode' on 'NodeFilter': The provided callback is not callable."
                ),
            );
            return Err(());
        };
        (function, filter.into())
    };
    let already_active = scope
        .get_slot::<NodeIteratorStore>()
        .and_then(|store| store.records.get(&record.id))
        .is_some_and(|record| record.active);
    if already_active {
        super::node::throw_dom_exception(
            scope,
            "InvalidStateError",
            &format!(
                "Failed to execute '{operation}' on 'NodeIterator': Filter function can't be recursive"
            ),
        );
        return Err(());
    }
    if let Some(iterator) = scope
        .get_slot_mut::<NodeIteratorStore>()
        .and_then(|store| store.records.get_mut(&record.id))
    {
        iterator.active = true;
    }
    let outcome = function
        .call(scope, receiver, &[node.into()])
        .and_then(|value| {
            super::node_filter::convert_filter_result(scope, value, operation, "NodeIterator").ok()
        });
    if let Some(iterator) = scope
        .get_slot_mut::<NodeIteratorStore>()
        .and_then(|store| store.records.get_mut(&record.id))
    {
        iterator.active = false;
    }
    outcome.map(|value| value == 1).ok_or(())
}

pub(crate) fn update_position(
    scope: &mut v8::PinScope<'_, '_>,
    iterator: v8::Local<'_, v8::Object>,
    reference: v8::Local<'_, v8::Object>,
    before: bool,
) {
    let reference = v8::Global::new(scope, reference);
    if let Some(record) = scope
        .get_slot_mut::<NodeIteratorStore>()
        .and_then(|store| store.records.get_mut(&iterator.get_identity_hash().get()))
    {
        record.reference = reference;
        record.pointer_before_reference_node = before;
    }
}

pub(crate) fn adjust_for_removal(
    scope: &mut v8::PinScope<'_, '_>,
    removed: v8::Local<'_, v8::Object>,
) {
    let records = scope
        .get_slot::<NodeIteratorStore>()
        .map(|store| {
            store
                .records
                .iter()
                .map(|(id, record)| (*id, record.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut updates = Vec::new();
    for (iterator_id, record) in records {
        let root = v8::Local::new(scope, &record.root);
        let reference = v8::Local::new(scope, &record.reference);
        if !super::node::is_descendant(scope, removed, reference)
            || super::node::is_descendant(scope, removed, root)
        {
            continue;
        }
        let mut nodes = Vec::new();
        collect_nodes(scope, root, &mut nodes);
        let Some(start) = nodes
            .iter()
            .position(|node| node.strict_equals(removed.into()))
        else {
            continue;
        };
        let mut after = start + 1;
        while after < nodes.len() && super::node::is_descendant(scope, removed, nodes[after]) {
            after += 1;
        }
        let (reference, before) = if record.pointer_before_reference_node && after < nodes.len() {
            (nodes[after], true)
        } else if start > 0 {
            (nodes[start - 1], false)
        } else {
            (root, true)
        };
        updates.push((iterator_id, v8::Global::new(scope, reference), before));
    }
    if let Some(store) = scope.get_slot_mut::<NodeIteratorStore>() {
        for (iterator_id, reference, before) in updates {
            if let Some(record) = store.records.get_mut(&iterator_id) {
                record.reference = reference;
                record.pointer_before_reference_node = before;
            }
        }
    }
}
