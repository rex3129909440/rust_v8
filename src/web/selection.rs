use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SelectionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
    document_selections: HashMap<i32, v8::Global<v8::Object>>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub anchor: Option<v8::Global<v8::Object>>,
    pub anchor_offset: u32,
    pub focus: Option<v8::Global<v8::Object>>,
    pub focus_offset: u32,
    pub ranges: Vec<v8::Global<v8::Object>>,
    pub direction: String,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SelectionStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Selection", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SelectionStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "Selection",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::selection_anchor_node_property::define(scope, p)?;
    super::selection_anchor_offset_property::define(scope, p)?;
    super::selection_focus_node_property::define(scope, p)?;
    super::selection_focus_offset_property::define(scope, p)?;
    super::selection_is_collapsed_property::define(scope, p)?;
    super::selection_range_count_property::define(scope, p)?;
    super::selection_type_property::define(scope, p)?;
    super::selection_direction_property::define(scope, p)?;
    super::selection_base_node_property::define(scope, p)?;
    super::selection_base_offset_property::define(scope, p)?;
    super::selection_extent_node_property::define(scope, p)?;
    super::selection_extent_offset_property::define(scope, p)?;
    super::selection_add_range::define(scope, p)?;
    super::selection_collapse::define(scope, p)?;
    super::selection_collapse_to_end::define(scope, p)?;
    super::selection_collapse_to_start::define(scope, p)?;
    super::selection_contains_node::define(scope, p)?;
    super::selection_delete_from_document::define(scope, p)?;
    super::selection_empty::define(scope, p)?;
    super::selection_extend::define(scope, p)?;
    super::selection_get_composed_ranges::define(scope, p)?;
    super::selection_get_range_at::define(scope, p)?;
    super::selection_modify::define(scope, p)?;
    super::selection_remove_all_ranges::define(scope, p)?;
    super::selection_remove_range::define(scope, p)?;
    super::selection_select_all_children::define(scope, p)?;
    super::selection_set_base_and_extent::define(scope, p)?;
    super::selection_set_position::define(scope, p)?;
    super::selection_to_string::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SelectionStore>()
        .ok_or_else(|| "Selection state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create Selection".to_owned());
    }
    scope
        .get_slot_mut::<SelectionStore>()
        .ok_or_else(|| "Selection state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            Record {
                anchor: None,
                anchor_offset: 0,
                focus: None,
                focus_offset: 0,
                ranges: Vec::new(),
                direction: "none".to_owned(),
            },
        );
    Ok(o)
}

pub(crate) fn for_document<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let identity = document.get_identity_hash().get();
    if let Some(selection) = scope
        .get_slot::<SelectionStore>()
        .and_then(|store| store.document_selections.get(&identity))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &selection));
    }
    let selection = create(scope)?;
    let stored = v8::Global::new(scope, selection);
    scope
        .get_slot_mut::<SelectionStore>()
        .ok_or_else(|| "Selection state was not prepared".to_owned())?
        .document_selections
        .insert(identity, stored);
    Ok(selection)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Selection': Illegal constructor",
    );
}
pub(crate) fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<SelectionStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(v) = scope
        .get_slot_mut::<SelectionStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn return_node(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    focus: bool,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = if focus { v.focus } else { v.anchor };
    if let Some(v) = value {
        r.set(v8::Local::new(scope, &v).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
pub(crate) fn valid_offset(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    offset: u32,
) -> bool {
    let Some(record) = super::node::record(scope, node) else {
        return false;
    };
    let maximum = if matches!(record.node_type, 3 | 4 | 7 | 8) {
        super::character_data::data_if_character(scope, node)
            .map(|data| data.encode_utf16().count())
            .unwrap_or(0)
    } else {
        record.children.len()
    };
    offset as usize <= maximum
}

pub(crate) fn selection_range(
    scope: &mut v8::PinScope<'_, '_>,
    start: v8::Local<'_, v8::Object>,
    start_offset: u32,
    end: v8::Local<'_, v8::Object>,
    end_offset: u32,
) -> Option<v8::Global<v8::Object>> {
    let document = super::node::record(scope, start).and_then(|record| {
        if record.node_type == 9 {
            Some(v8::Global::new(scope, start))
        } else {
            record.owner_document
        }
    })?;
    let document = v8::Local::new(scope, &document);
    let range = super::range::create(scope, document).ok()?;
    let order = super::range::compare_boundaries(scope, start, start_offset, end, end_offset);
    let (start, start_offset, end, end_offset) = if order.is_some_and(|order| order > 0) {
        (end, end_offset, start, start_offset)
    } else {
        (start, start_offset, end, end_offset)
    };
    let start_container = v8::Global::new(scope, start);
    let end_container = v8::Global::new(scope, end);
    super::abstract_range::update(scope, range, |record| {
        record.start_container = start_container;
        record.start_offset = start_offset;
        record.end_container = end_container;
        record.end_offset = end_offset;
    });
    Some(v8::Global::new(scope, range))
}

pub(crate) fn direction_between(
    scope: &v8::PinScope<'_, '_>,
    anchor: v8::Local<'_, v8::Object>,
    anchor_offset: u32,
    focus: v8::Local<'_, v8::Object>,
    focus_offset: u32,
) -> String {
    match super::range::compare_boundaries(scope, anchor, anchor_offset, focus, focus_offset) {
        Some(order) if order > 0 => "backward".to_owned(),
        Some(order) if order < 0 => "forward".to_owned(),
        _ => "none".to_owned(),
    }
}
