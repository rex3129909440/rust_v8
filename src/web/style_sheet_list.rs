use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StyleSheetListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StyleSheetListStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StyleSheetList", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StyleSheetListStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StyleSheetList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "length", get_length)?;
    crate::webidl::define_method(scope, p, "item", 1, item)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_indexed_iterator(scope, p)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StyleSheetListStore>()
        .ok_or_else(|| "StyleSheetList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create StyleSheetList".to_owned());
    }
    let mut stored = Vec::new();
    for (i, v) in values.into_iter().enumerate() {
        let k = crate::webidl::string(scope, &i.to_string())?;
        let _ = o.define_own_property(scope, k.into(), v.into(), v8::PropertyAttribute::READ_ONLY);
        stored.push(v8::Global::new(scope, v));
    }
    scope
        .get_slot_mut::<StyleSheetListStore>()
        .ok_or_else(|| "StyleSheetList state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), stored);
    Ok(o)
}

pub(crate) fn replace_values(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    values: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    let Some(old_length) = record(scope, object).map(|values| values.len()) else {
        return false;
    };
    for index in 0..old_length {
        let _ = object.delete_index(scope, index as u32);
    }
    let mut stored = Vec::with_capacity(values.len());
    for (index, value) in values.into_iter().enumerate() {
        let Some(key) = v8::String::new(scope, &index.to_string()) else {
            return false;
        };
        if object.define_own_property(
            scope,
            key.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY,
        ) != Some(true)
        {
            return false;
        }
        stored.push(v8::Global::new(scope, value));
    }
    if let Some(record) = scope
        .get_slot_mut::<StyleSheetListStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        *record = stored;
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
        "Failed to construct 'StyleSheetList': Illegal constructor",
    );
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<StyleSheetListStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, v.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let i = a.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(v) = v.get(i) {
        r.set(v8::Local::new(scope, v).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
