use std::collections::HashMap;

const SHOW_ALL: u32 = u32::MAX;
pub(crate) const FILTER_ACCEPT: i32 = 1;
pub(crate) const FILTER_REJECT: i32 = 2;
const FILTER_SKIP: i32 = 3;

#[derive(Default)]
pub(crate) struct TreeWalkerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TreeWalkerRecord>,
}

#[derive(Clone)]
pub(crate) struct TreeWalkerRecord {
    pub id: i32,
    pub root: v8::Global<v8::Object>,
    pub what_to_show: u32,
    pub filter: Option<v8::Global<v8::Object>>,
    pub current: v8::Global<v8::Object>,
    pub active: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TreeWalkerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TreeWalker", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TreeWalkerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TreeWalker",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::tree_walker_root_property::define(scope, prototype)?;
    super::tree_walker_what_to_show_property::define(scope, prototype)?;
    super::tree_walker_filter_property::define(scope, prototype)?;
    super::tree_walker_current_node_property::define(scope, prototype)?;
    super::tree_walker_first_child::define(scope, prototype)?;
    super::tree_walker_last_child::define(scope, prototype)?;
    super::tree_walker_next_node::define(scope, prototype)?;
    super::tree_walker_next_sibling::define(scope, prototype)?;
    super::tree_walker_parent_node::define(scope, prototype)?;
    super::tree_walker_previous_node::define(scope, prototype)?;
    super::tree_walker_previous_sibling::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TreeWalkerStore>()
        .ok_or_else(|| "TreeWalker state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    root: v8::Local<'_, v8::Object>,
    what_to_show: Option<u32>,
    filter: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let walker = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, walker, prototype.into()) != Some(true) {
        return Err("cannot create TreeWalker".to_owned());
    }
    let root_global = v8::Global::new(scope, root);
    let current_global = v8::Global::new(scope, root);
    let filter_global = filter
        .filter(|value| !value.is_null() && !value.is_undefined())
        .map(|value| v8::Global::new(scope, value));
    let id = walker.get_identity_hash().get();
    scope
        .get_slot_mut::<TreeWalkerStore>()
        .ok_or_else(|| "TreeWalker state was not prepared".to_owned())?
        .records
        .insert(
            id,
            TreeWalkerRecord {
                id,
                root: root_global,
                what_to_show: what_to_show.unwrap_or(SHOW_ALL),
                filter: filter_global,
                current: current_global,
                active: false,
            },
        );
    Ok(walker)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TreeWalker': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TreeWalkerRecord> {
    scope
        .get_slot::<TreeWalkerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn set_current(
    scope: &mut v8::PinScope<'_, '_>,
    walker: v8::Local<'_, v8::Object>,
    node: v8::Local<'_, v8::Object>,
) {
    let node = v8::Global::new(scope, node);
    if let Some(record) = scope
        .get_slot_mut::<TreeWalkerStore>()
        .and_then(|store| store.records.get_mut(&walker.get_identity_hash().get()))
    {
        record.current = node;
    }
}

pub(crate) fn direct_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::node::parent(scope, object)
}

pub(crate) fn filter_status(
    scope: &mut v8::PinScope<'_, '_>,
    walker_record: &TreeWalkerRecord,
    node: v8::Local<'_, v8::Object>,
    operation: &str,
) -> Result<i32, ()> {
    let node_type = super::node::record(scope, node)
        .map(|record| record.node_type)
        .unwrap_or(0);
    if node_type > 0
        && node_type <= 32
        && walker_record.what_to_show != SHOW_ALL
        && walker_record.what_to_show & (1_u32 << (node_type - 1)) == 0
    {
        return Ok(FILTER_SKIP);
    }
    let Some(filter) = walker_record.filter.as_ref() else {
        return Ok(FILTER_ACCEPT);
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
                    "Failed to execute '{operation}' on 'TreeWalker': Failed to execute 'acceptNode' on 'NodeFilter': The provided callback is not callable."
                ),
            );
            return Err(());
        };
        (function, filter.into())
    };
    let already_active = scope
        .get_slot::<TreeWalkerStore>()
        .and_then(|store| store.records.get(&walker_record.id))
        .is_some_and(|record| record.active);
    if already_active {
        super::node::throw_dom_exception(
            scope,
            "InvalidStateError",
            &format!(
                "Failed to execute '{operation}' on 'TreeWalker': Filter function can't be recursive"
            ),
        );
        return Err(());
    }
    if let Some(record) = scope
        .get_slot_mut::<TreeWalkerStore>()
        .and_then(|store| store.records.get_mut(&walker_record.id))
    {
        record.active = true;
    }
    let outcome = function
        .call(scope, receiver, &[node.into()])
        .and_then(|value| {
            super::node_filter::convert_filter_result(scope, value, operation, "TreeWalker").ok()
        });
    if let Some(record) = scope
        .get_slot_mut::<TreeWalkerStore>()
        .and_then(|store| store.records.get_mut(&walker_record.id))
    {
        record.active = false;
    }
    outcome.ok_or(())
}

pub(crate) fn first_child<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::node::children(scope, node).into_iter().next()
}

pub(crate) fn last_child<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::node::children(scope, node).into_iter().next_back()
}

pub(crate) fn next_sibling<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = direct_parent(scope, node)?;
    let children = super::node::children(scope, parent);
    let index = children
        .iter()
        .position(|child| child.strict_equals(node.into()))?;
    children.get(index + 1).copied()
}

pub(crate) fn previous_sibling<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = direct_parent(scope, node)?;
    let children = super::node::children(scope, parent);
    let index = children
        .iter()
        .position(|child| child.strict_equals(node.into()))?;
    index
        .checked_sub(1)
        .and_then(|index| children.get(index))
        .copied()
}

pub(crate) fn traverse_children(
    scope: &mut v8::PinScope<'_, '_>,
    record: &TreeWalkerRecord,
    forward: bool,
    operation: &str,
) -> Result<Option<v8::Global<v8::Object>>, ()> {
    let boundary = v8::Local::new(scope, &record.current);
    let mut node = boundary;
    let mut candidate = if forward {
        first_child(scope, node)
    } else {
        last_child(scope, node)
    };
    while let Some(next) = candidate {
        node = next;
        let status = filter_status(scope, record, node, operation)?;
        if status == FILTER_ACCEPT {
            return Ok(Some(v8::Global::new(scope, node)));
        }
        if status != FILTER_REJECT {
            candidate = if forward {
                first_child(scope, node)
            } else {
                last_child(scope, node)
            };
            if candidate.is_some() {
                continue;
            }
        }
        loop {
            candidate = if forward {
                next_sibling(scope, node)
            } else {
                previous_sibling(scope, node)
            };
            if candidate.is_some() {
                break;
            }
            let Some(parent) = direct_parent(scope, node) else {
                return Ok(None);
            };
            if parent.strict_equals(boundary.into()) {
                return Ok(None);
            }
            node = parent;
        }
    }
    Ok(None)
}

pub(crate) fn traverse_siblings(
    scope: &mut v8::PinScope<'_, '_>,
    record: &TreeWalkerRecord,
    forward: bool,
    operation: &str,
) -> Result<Option<v8::Global<v8::Object>>, ()> {
    let root = v8::Local::new(scope, &record.root);
    let mut node = v8::Local::new(scope, &record.current);
    if node.strict_equals(root.into()) {
        return Ok(None);
    }
    loop {
        let mut sibling = if forward {
            next_sibling(scope, node)
        } else {
            previous_sibling(scope, node)
        };
        while let Some(next) = sibling {
            if let Some(candidate) =
                visible_in_sibling_branch(scope, record, next, forward, operation)?
            {
                return Ok(Some(candidate));
            }
            sibling = if forward {
                next_sibling(scope, next)
            } else {
                previous_sibling(scope, next)
            };
        }
        let Some(parent) = direct_parent(scope, node) else {
            return Ok(None);
        };
        if parent.strict_equals(root.into()) {
            return Ok(None);
        }
        node = parent;
        let status = filter_status(scope, record, node, operation)?;
        if status == FILTER_ACCEPT || status == FILTER_REJECT {
            return Ok(None);
        }
    }
}

fn visible_in_sibling_branch(
    scope: &mut v8::PinScope<'_, '_>,
    record: &TreeWalkerRecord,
    node: v8::Local<'_, v8::Object>,
    forward: bool,
    operation: &str,
) -> Result<Option<v8::Global<v8::Object>>, ()> {
    let status = filter_status(scope, record, node, operation)?;
    if status == FILTER_ACCEPT {
        return Ok(Some(v8::Global::new(scope, node)));
    }
    if status == FILTER_REJECT {
        return Ok(None);
    }
    let children = super::node::children(scope, node);
    if forward {
        for child in children {
            if let Some(candidate) =
                visible_in_sibling_branch(scope, record, child, forward, operation)?
            {
                return Ok(Some(candidate));
            }
        }
    } else {
        for child in children.into_iter().rev() {
            if let Some(candidate) =
                visible_in_sibling_branch(scope, record, child, forward, operation)?
            {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}

pub(crate) fn traverse_parent(
    scope: &mut v8::PinScope<'_, '_>,
    record: &TreeWalkerRecord,
    operation: &str,
) -> Result<Option<v8::Global<v8::Object>>, ()> {
    let root = v8::Local::new(scope, &record.root);
    let mut node = v8::Local::new(scope, &record.current);
    if node.strict_equals(root.into()) {
        return Ok(None);
    }
    while let Some(parent) = direct_parent(scope, node) {
        node = parent;
        let accepted = filter_status(scope, record, node, operation)? == FILTER_ACCEPT;
        if accepted {
            return Ok(Some(v8::Global::new(scope, node)));
        }
        if node.strict_equals(root.into()) {
            return Ok(None);
        }
    }
    Ok(None)
}

pub(crate) fn traverse_next(
    scope: &mut v8::PinScope<'_, '_>,
    record: &TreeWalkerRecord,
    operation: &str,
) -> Result<Option<v8::Global<v8::Object>>, ()> {
    let root = v8::Local::new(scope, &record.root);
    let mut node = v8::Local::new(scope, &record.current);
    let mut status = FILTER_ACCEPT;
    loop {
        while status != FILTER_REJECT {
            let Some(child) = first_child(scope, node) else {
                break;
            };
            node = child;
            status = filter_status(scope, record, node, operation)?;
            if status == FILTER_ACCEPT {
                return Ok(Some(v8::Global::new(scope, node)));
            }
        }
        let mut temporary = Some(node);
        let mut sibling = None;
        while let Some(value) = temporary {
            if value.strict_equals(root.into()) {
                return Ok(None);
            }
            sibling = next_sibling(scope, value);
            if sibling.is_some() {
                break;
            }
            temporary = direct_parent(scope, value);
        }
        let Some(next) = sibling else {
            return Ok(None);
        };
        node = next;
        status = filter_status(scope, record, node, operation)?;
        if status == FILTER_ACCEPT {
            return Ok(Some(v8::Global::new(scope, node)));
        }
    }
}

pub(crate) fn traverse_previous(
    scope: &mut v8::PinScope<'_, '_>,
    record: &TreeWalkerRecord,
    operation: &str,
) -> Result<Option<v8::Global<v8::Object>>, ()> {
    let root = v8::Local::new(scope, &record.root);
    let mut node = v8::Local::new(scope, &record.current);
    while !node.strict_equals(root.into()) {
        let mut sibling = previous_sibling(scope, node);
        while let Some(previous) = sibling {
            node = previous;
            let mut status = filter_status(scope, record, node, operation)?;
            while status != FILTER_REJECT {
                let Some(child) = last_child(scope, node) else {
                    break;
                };
                node = child;
                status = filter_status(scope, record, node, operation)?;
            }
            if status == FILTER_ACCEPT {
                return Ok(Some(v8::Global::new(scope, node)));
            }
            sibling = previous_sibling(scope, node);
        }
        let Some(parent) = direct_parent(scope, node) else {
            return Ok(None);
        };
        node = parent;
        if node.strict_equals(root.into()) {
            return if filter_status(scope, record, node, operation)? == FILTER_ACCEPT {
                Ok(Some(v8::Global::new(scope, node)))
            } else {
                Ok(None)
            };
        }
        if filter_status(scope, record, node, operation)? == FILTER_ACCEPT {
            return Ok(Some(v8::Global::new(scope, node)));
        }
    }
    Ok(None)
}

pub(crate) fn return_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    walker: v8::Local<'_, v8::Object>,
    candidate: Option<v8::Local<'_, v8::Object>>,
    mut result: v8::ReturnValue<'_>,
) {
    match candidate {
        Some(candidate) => {
            set_current(scope, walker, candidate);
            result.set(candidate.into());
        }
        None => result.set(v8::null(scope).into()),
    }
}
