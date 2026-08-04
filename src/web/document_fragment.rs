use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DocumentFragmentStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashMap<i32, FragmentRecord>,
}
#[derive(Clone, Default)]
struct FragmentRecord {
    children: Option<v8::Global<v8::Object>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentFragmentStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DocumentFragment", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<DocumentFragmentStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "DocumentFragment",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::document_fragment_children_property::define(scope, p)?;
    super::document_fragment_first_element_child_property::define(scope, p)?;
    super::document_fragment_last_element_child_property::define(scope, p)?;
    super::document_fragment_child_element_count_property::define(scope, p)?;
    super::parent_node_append::define(scope, p)?;
    super::document_fragment_get_element_by_id::define(scope, p)?;
    super::parent_node_move_before::define(scope, p)?;
    super::parent_node_prepend::define(scope, p)?;
    super::document_fragment_query_selector::define(scope, p)?;
    super::document_fragment_query_selector_all::define(scope, p)?;
    super::parent_node_replace_children::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let unscopables = crate::webidl::new_unscopables(scope)?;
    crate::webidl::define_unscopable(scope, unscopables, "append")?;
    crate::webidl::define_unscopable(scope, unscopables, "prepend")?;
    crate::webidl::define_unscopable(scope, unscopables, "replaceChildren")?;
    crate::webidl::attach_unscopables(scope, p, unscopables)?;
    let parent = super::node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<DocumentFragmentStore>()
        .ok_or_else(|| "DocumentFragment state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'DocumentFragment': use new");
        return;
    }
    attach(scope, a.this());
    r.set(a.this().into())
}
pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) {
    super::node::attach(scope, o, 11, "#document-fragment".to_owned(), None);
    scope
        .get_slot_mut::<DocumentFragmentStore>()
        .expect("DocumentFragment state")
        .instances
        .insert(o.get_identity_hash().get(), FragmentRecord::default());
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create DocumentFragment".to_owned());
    }
    attach(scope, o);
    Ok(o)
}
pub(crate) fn valid(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<DocumentFragmentStore>()
        .is_some_and(|s| s.instances.contains_key(&o.get_identity_hash().get()))
}
pub(crate) fn elements<'s>(
    scope: &v8::PinScope<'s, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::node::children(scope, o)
        .into_iter()
        .filter(|v| super::node::record(scope, *v).is_some_and(|r| r.node_type == 1))
        .collect()
}

pub(crate) fn cached_children(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<DocumentFragmentStore>()?
        .instances
        .get(&object.get_identity_hash().get())?
        .children
        .clone()
}

pub(crate) fn cache_children(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    collection: v8::Global<v8::Object>,
) {
    if let Some(record) = scope
        .get_slot_mut::<DocumentFragmentStore>()
        .and_then(|store| store.instances.get_mut(&object.get_identity_hash().get()))
    {
        record.children = Some(collection);
    }
}
