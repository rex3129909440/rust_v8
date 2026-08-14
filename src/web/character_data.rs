use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CharacterDataStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CharacterDataStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CharacterData", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<CharacterDataStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "CharacterData",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::character_data_data_property::define(scope, p)?;
    super::character_data_length_property::define(scope, p)?;
    super::character_data_previous_element_sibling_property::define(scope, p)?;
    super::character_data_next_element_sibling_property::define(scope, p)?;
    super::character_data_after::define(scope, p)?;
    super::character_data_append_data::define(scope, p)?;
    super::character_data_before::define(scope, p)?;
    super::character_data_delete_data::define(scope, p)?;
    super::character_data_insert_data::define(scope, p)?;
    super::character_data_remove::define(scope, p)?;
    super::character_data_replace_data::define(scope, p)?;
    super::character_data_replace_with::define(scope, p)?;
    super::character_data_substring_data::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let unscopables = crate::webidl::new_unscopables(scope)?;
    crate::webidl::define_unscopable(scope, unscopables, "after")?;
    crate::webidl::define_unscopable(scope, unscopables, "before")?;
    crate::webidl::define_unscopable(scope, unscopables, "remove")?;
    crate::webidl::define_unscopable(scope, unscopables, "replaceWith")?;
    crate::webidl::attach_unscopables(scope, p, unscopables)?;
    let node = super::node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, node)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<CharacterDataStore>()
        .ok_or_else(|| "CharacterData state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CharacterData': Illegal constructor",
    );
}
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    data: String,
) {
    scope
        .get_slot_mut::<CharacterDataStore>()
        .expect("CharacterData state")
        .records
        .insert(object.get_identity_hash().get(), data);
}
pub(crate) fn data_if_character(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    scope
        .get_slot::<CharacterDataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn set_data_if_character(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    data: String,
) -> bool {
    let Some(old_data) = data_if_character(scope, object) else {
        return false;
    };
    let old_length = old_data.encode_utf16().count() as u32;
    let new_length = data.encode_utf16().count() as u32;
    if !set_data_without_range_adjustment(scope, object, data) {
        return false;
    }
    super::abstract_range::adjust_for_character_data(scope, object, 0, old_length, new_length);
    true
}

pub(crate) fn set_data_without_range_adjustment(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    data: String,
) -> bool {
    let old_value = {
        let Some(value) = scope
            .get_slot_mut::<CharacterDataStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        else {
            return false;
        };
        let old_value = value.clone();
        *value = data;
        old_value
    };
    super::mutation_observer::enqueue_character_data_change(scope, object, Some(old_value));
    true
}
fn utf16(data: &str) -> Vec<u16> {
    data.encode_utf16().collect()
}
fn replace_units(data: &str, offset: u32, count: u32, replacement: &str) -> Result<String, String> {
    let mut units = utf16(data);
    let start = offset as usize;
    if start > units.len() {
        return Err("The offset is larger than the data length".to_owned());
    }
    let end = start.saturating_add(count as usize).min(units.len());
    units.splice(start..end, replacement.encode_utf16());
    Ok(String::from_utf16_lossy(&units))
}
pub(crate) enum EditError {
    IllegalInvocation,
    IndexSize,
}

pub(crate) fn require_arguments(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    method: &str,
    required: i32,
) -> bool {
    if arguments.length() >= required {
        return true;
    }
    let noun = if required == 1 {
        "argument"
    } else {
        "arguments"
    };
    crate::webidl::throw_type_error(
        scope,
        &format!(
            "Failed to execute '{method}' on 'CharacterData': {required} {noun} required, but only {} present.",
            arguments.length()
        ),
    );
    false
}

pub(crate) fn unsigned_long_argument(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    method: &str,
) -> Option<u32> {
    if value.is_symbol() || value.is_big_int() {
        let kind = if value.is_symbol() {
            "Symbol"
        } else {
            "BigInt"
        };
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'CharacterData': Cannot convert a {kind} value to a number"
            ),
        );
        return None;
    }
    value.uint32_value(scope)
}

pub(crate) fn string_argument(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    context: &str,
) -> Option<String> {
    if value.is_symbol() {
        crate::webidl::throw_type_error(
            scope,
            &format!("{context}: Cannot convert a Symbol value to a string"),
        );
        return None;
    }
    value
        .to_string(scope)
        .map(|value| value.to_rust_string_lossy(scope))
}

pub(crate) fn throw_offset_error(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    method: &str,
    offset: u32,
) {
    let length = data_if_character(scope, object)
        .map(|value| value.encode_utf16().count())
        .unwrap_or(0);
    super::node::throw_dom_exception(
        scope,
        "IndexSizeError",
        &format!(
            "Failed to execute '{method}' on 'CharacterData': The offset {offset} is greater than the node's length ({length})."
        ),
    );
}
pub(crate) fn replace_data_units(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    offset: u32,
    count: u32,
    replacement: &str,
) -> Result<(), EditError> {
    let data = data_if_character(scope, object).ok_or(EditError::IllegalInvocation)?;
    let original_length = data.encode_utf16().count() as u32;
    let value =
        replace_units(&data, offset, count, replacement).map_err(|_| EditError::IndexSize)?;
    let removed_count = count.min(original_length.saturating_sub(offset));
    let inserted_count = replacement.encode_utf16().count() as u32;
    set_data_without_range_adjustment(scope, object, value);
    super::abstract_range::adjust_for_character_data(
        scope,
        object,
        offset,
        removed_count,
        inserted_count,
    );
    Ok(())
}
pub(crate) fn element_sibling<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    next: bool,
) -> Option<v8::Local<'s, v8::Object>> {
    let parent = super::node::parent(scope, object)?;
    let values = super::node::children(scope, parent);
    let id = object.get_identity_hash().get();
    let index = values
        .iter()
        .position(|v| v.get_identity_hash().get() == id)?;
    if next {
        values
            .into_iter()
            .skip(index + 1)
            .find(|v| super::node::record(scope, *v).is_some_and(|r| r.node_type == 1))
    } else {
        values
            .into_iter()
            .take(index)
            .rev()
            .find(|v| super::node::record(scope, *v).is_some_and(|r| r.node_type == 1))
    }
}
