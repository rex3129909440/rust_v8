use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AbstractRangeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RangeRecord>,
}
#[derive(Clone)]
pub(crate) struct RangeRecord {
    pub start_container: v8::Global<v8::Object>,
    pub start_offset: u32,
    pub end_container: v8::Global<v8::Object>,
    pub end_offset: u32,
    pub live: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AbstractRangeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AbstractRange", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AbstractRangeStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "AbstractRange",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::abstract_range_start_container_property::define(scope, p)?;
    super::abstract_range_start_offset_property::define(scope, p)?;
    super::abstract_range_end_container_property::define(scope, p)?;
    super::abstract_range_end_offset_property::define(scope, p)?;
    super::abstract_range_collapsed_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<AbstractRangeStore>()
        .ok_or_else(|| "AbstractRange state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    start: v8::Local<'_, v8::Object>,
    start_offset: u32,
    end: v8::Local<'_, v8::Object>,
    end_offset: u32,
) {
    attach_with_liveness(scope, o, start, start_offset, end, end_offset, false);
}

pub(crate) fn attach_live(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    start: v8::Local<'_, v8::Object>,
    start_offset: u32,
    end: v8::Local<'_, v8::Object>,
    end_offset: u32,
) {
    attach_with_liveness(scope, o, start, start_offset, end, end_offset, true);
}

fn attach_with_liveness(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    start: v8::Local<'_, v8::Object>,
    start_offset: u32,
    end: v8::Local<'_, v8::Object>,
    end_offset: u32,
    live: bool,
) {
    let record = RangeRecord {
        start_container: v8::Global::new(scope, start),
        start_offset,
        end_container: v8::Global::new(scope, end),
        end_offset,
        live,
    };
    scope
        .get_slot_mut::<AbstractRangeStore>()
        .expect("AbstractRange state")
        .records
        .insert(o.get_identity_hash().get(), record);
}

pub(crate) fn adjust_for_insertion(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    index: usize,
    count: usize,
) {
    let parent_id = parent.get_identity_hash().get();
    let updates = scope
        .get_slot::<AbstractRangeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.live)
                .map(|(range_id, record)| {
                    let start_id = v8::Local::new(scope, &record.start_container)
                        .get_identity_hash()
                        .get();
                    let end_id = v8::Local::new(scope, &record.end_container)
                        .get_identity_hash()
                        .get();
                    (*range_id, start_id == parent_id, end_id == parent_id)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(store) = scope.get_slot_mut::<AbstractRangeStore>() {
        for (range_id, start_matches, end_matches) in updates {
            let Some(record) = store.records.get_mut(&range_id) else {
                continue;
            };
            if start_matches && record.start_offset as usize > index {
                record.start_offset = record.start_offset.saturating_add(count as u32);
            }
            if end_matches && record.end_offset as usize > index {
                record.end_offset = record.end_offset.saturating_add(count as u32);
            }
        }
    }
}

pub(crate) fn adjust_for_removal(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    removed: v8::Local<'_, v8::Object>,
    index: usize,
) {
    fn collect(
        scope: &v8::PinScope<'_, '_>,
        node: v8::Local<'_, v8::Object>,
        identities: &mut std::collections::HashSet<i32>,
    ) {
        identities.insert(node.get_identity_hash().get());
        for child in super::node::children(scope, node) {
            collect(scope, child, identities);
        }
    }
    let mut removed_ids = std::collections::HashSet::new();
    collect(scope, removed, &mut removed_ids);
    let parent_id = parent.get_identity_hash().get();
    let parent_global = v8::Global::new(scope, parent);
    let updates = scope
        .get_slot::<AbstractRangeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.live)
                .map(|(range_id, record)| {
                    let start_id = v8::Local::new(scope, &record.start_container)
                        .get_identity_hash()
                        .get();
                    let end_id = v8::Local::new(scope, &record.end_container)
                        .get_identity_hash()
                        .get();
                    (
                        *range_id,
                        removed_ids.contains(&start_id),
                        removed_ids.contains(&end_id),
                        start_id == parent_id,
                        end_id == parent_id,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(store) = scope.get_slot_mut::<AbstractRangeStore>() {
        for (range_id, start_removed, end_removed, start_parent, end_parent) in updates {
            let Some(record) = store.records.get_mut(&range_id) else {
                continue;
            };
            if start_removed {
                record.start_container = parent_global.clone();
                record.start_offset = index as u32;
            } else if start_parent && record.start_offset as usize > index {
                record.start_offset = record.start_offset.saturating_sub(1);
            }
            if end_removed {
                record.end_container = parent_global.clone();
                record.end_offset = index as u32;
            } else if end_parent && record.end_offset as usize > index {
                record.end_offset = record.end_offset.saturating_sub(1);
            }
        }
    }
}

pub(crate) fn adjust_for_character_data(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    offset: u32,
    removed_count: u32,
    inserted_count: u32,
) {
    let node_id = node.get_identity_hash().get();
    let updates = scope
        .get_slot::<AbstractRangeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.live)
                .map(|(range_id, record)| {
                    let start_id = v8::Local::new(scope, &record.start_container)
                        .get_identity_hash()
                        .get();
                    let end_id = v8::Local::new(scope, &record.end_container)
                        .get_identity_hash()
                        .get();
                    (*range_id, start_id == node_id, end_id == node_id)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let removed_end = offset.saturating_add(removed_count);
    let adjust = |boundary: u32| {
        if boundary > removed_end {
            boundary
                .saturating_sub(removed_count)
                .saturating_add(inserted_count)
        } else if boundary > offset {
            offset
        } else {
            boundary
        }
    };
    if let Some(store) = scope.get_slot_mut::<AbstractRangeStore>() {
        for (range_id, start_matches, end_matches) in updates {
            let Some(record) = store.records.get_mut(&range_id) else {
                continue;
            };
            if start_matches {
                record.start_offset = adjust(record.start_offset);
            }
            if end_matches {
                record.end_offset = adjust(record.end_offset);
            }
        }
    }
}

pub(crate) fn adjust_for_split_text(
    scope: &mut v8::PinScope<'_, '_>,
    original: v8::Local<'_, v8::Object>,
    new_text: v8::Local<'_, v8::Object>,
    offset: u32,
) {
    let original_id = original.get_identity_hash().get();
    let new_text = v8::Global::new(scope, new_text);
    let parent_and_offset = super::node::parent(scope, original).and_then(|parent| {
        super::node::children(scope, parent)
            .iter()
            .position(|node| node.strict_equals(original.into()))
            .map(|index| (parent.get_identity_hash().get(), index as u32 + 1))
    });
    let updates = scope
        .get_slot::<AbstractRangeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.live)
                .map(|(range_id, record)| {
                    let start_id = v8::Local::new(scope, &record.start_container)
                        .get_identity_hash()
                        .get();
                    let end_id = v8::Local::new(scope, &record.end_container)
                        .get_identity_hash()
                        .get();
                    (
                        *range_id,
                        start_id == original_id,
                        end_id == original_id,
                        parent_and_offset.is_some_and(|(parent_id, _)| start_id == parent_id),
                        parent_and_offset.is_some_and(|(parent_id, _)| end_id == parent_id),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(store) = scope.get_slot_mut::<AbstractRangeStore>() {
        for (range_id, start_matches, end_matches, start_parent, end_parent) in updates {
            let Some(record) = store.records.get_mut(&range_id) else {
                continue;
            };
            if start_matches && record.start_offset > offset {
                record.start_container = new_text.clone();
                record.start_offset -= offset;
            }
            if end_matches && record.end_offset > offset {
                record.end_container = new_text.clone();
                record.end_offset -= offset;
            }
            if let Some((_, split_offset)) = parent_and_offset {
                if start_parent && record.start_offset == split_offset {
                    record.start_offset = record.start_offset.saturating_add(1);
                }
                if end_parent && record.end_offset == split_offset {
                    record.end_offset = record.end_offset.saturating_add(1);
                }
            }
        }
    }
}

pub(crate) fn adjust_for_text_merge(
    scope: &mut v8::PinScope<'_, '_>,
    removed: v8::Local<'_, v8::Object>,
    kept: v8::Local<'_, v8::Object>,
    kept_length: u32,
) {
    let removed_id = removed.get_identity_hash().get();
    let kept = v8::Global::new(scope, kept);
    let parent_and_index = super::node::parent(scope, removed).and_then(|parent| {
        super::node::children(scope, parent)
            .iter()
            .position(|node| node.strict_equals(removed.into()))
            .map(|index| (v8::Global::new(scope, parent), index as u32))
    });
    let updates = scope
        .get_slot::<AbstractRangeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.live)
                .map(|(range_id, record)| {
                    let start_id = v8::Local::new(scope, &record.start_container)
                        .get_identity_hash()
                        .get();
                    let end_id = v8::Local::new(scope, &record.end_container)
                        .get_identity_hash()
                        .get();
                    let parent_id = parent_and_index
                        .as_ref()
                        .map(|(parent, _)| v8::Local::new(scope, parent).get_identity_hash().get());
                    (
                        *range_id,
                        start_id == removed_id,
                        end_id == removed_id,
                        parent_id == Some(start_id),
                        parent_id == Some(end_id),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(store) = scope.get_slot_mut::<AbstractRangeStore>() {
        for (range_id, start_matches, end_matches, start_parent, end_parent) in updates {
            let Some(record) = store.records.get_mut(&range_id) else {
                continue;
            };
            if start_matches {
                record.start_container = kept.clone();
                record.start_offset = kept_length.saturating_add(record.start_offset);
            }
            if end_matches {
                record.end_container = kept.clone();
                record.end_offset = kept_length.saturating_add(record.end_offset);
            }
            if let Some((_, removed_index)) = parent_and_index.as_ref() {
                if start_parent && record.start_offset == *removed_index {
                    record.start_container = kept.clone();
                    record.start_offset = kept_length;
                }
                if end_parent && record.end_offset == *removed_index {
                    record.end_container = kept.clone();
                    record.end_offset = kept_length;
                }
            }
        }
    }
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<RangeRecord> {
    scope
        .get_slot::<AbstractRangeStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut RangeRecord),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<AbstractRangeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
        true
    } else {
        false
    }
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AbstractRange': Illegal constructor",
    );
}
