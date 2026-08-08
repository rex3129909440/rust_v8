use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextTrackCueListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextTrackCueListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextTrackCueList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextTrackCueListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TextTrackCueList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "getCueById", 1, get_cue_by_id)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TextTrackCueListStore>()
        .ok_or_else(|| "TextTrackCueList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create TextTrackCueList".to_owned());
    }
    scope
        .get_slot_mut::<TextTrackCueListStore>()
        .ok_or_else(|| "TextTrackCueList state was not prepared".to_owned())?
        .records
        .insert(list.get_identity_hash().get(), Vec::new());
    Ok(list)
}

pub(crate) fn add(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    cue: v8::Local<'_, v8::Object>,
) -> bool {
    let identity = list.get_identity_hash().get();
    let duplicate = scope
        .get_slot::<TextTrackCueListStore>()
        .and_then(|store| store.records.get(&identity))
        .is_some_and(|values| {
            values
                .iter()
                .any(|value| v8::Local::new(scope, value).strict_equals(cue.into()))
        });
    if duplicate {
        return false;
    }
    let index = scope
        .get_slot::<TextTrackCueListStore>()
        .and_then(|store| store.records.get(&identity))
        .map(Vec::len)
        .unwrap_or(0);
    let cue_global = v8::Global::new(scope, cue);
    let Some(values) = scope
        .get_slot_mut::<TextTrackCueListStore>()
        .and_then(|store| store.records.get_mut(&identity))
    else {
        return false;
    };
    values.push(cue_global);
    let Some(index_name) = v8::String::new(scope, &index.to_string()) else {
        return false;
    };
    let _ = list.define_own_property(
        scope,
        index_name.into(),
        cue.into(),
        v8::PropertyAttribute::READ_ONLY,
    );
    true
}

pub(crate) fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    list: v8::Local<'_, v8::Object>,
    cue: v8::Local<'_, v8::Object>,
) -> bool {
    let identity = list.get_identity_hash().get();
    let snapshot = scope
        .get_slot::<TextTrackCueListStore>()
        .and_then(|store| store.records.get(&identity))
        .cloned()
        .unwrap_or_default();
    let Some(position) = snapshot
        .iter()
        .position(|value| v8::Local::new(scope, value).strict_equals(cue.into()))
    else {
        return false;
    };
    if let Some(values) = scope
        .get_slot_mut::<TextTrackCueListStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        values.remove(position);
    }
    let old_last = snapshot.len().saturating_sub(1) as u32;
    for index in position as u32..old_last {
        let replacement = v8::Local::new(scope, &snapshot[index as usize + 1]);
        let Some(index_name) = v8::String::new(scope, &index.to_string()) else {
            return false;
        };
        let _ = list.define_own_property(
            scope,
            index_name.into(),
            replacement.into(),
            v8::PropertyAttribute::READ_ONLY,
        );
    }
    if let Some(last_name) = v8::String::new(scope, &old_last.to_string()) {
        let _ = list.delete(scope, last_name.into());
    }
    true
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TextTrackCueList': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<TextTrackCueListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, values.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_cue_by_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let wanted = crate::webidl::value_to_string(scope, arguments.get(0));
    for value in values {
        let cue = v8::Local::new(scope, &value);
        if super::text_track_cue::id(scope, cue).is_some_and(|id| id == wanted) {
            result.set(cue.into());
            return;
        }
    }
    result.set(v8::null(scope).into());
}
