use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TouchListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TouchListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TouchList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TouchListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TouchList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TouchListStore>()
        .ok_or_else(|| "TouchList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create TouchList".to_owned());
    }
    let mut stored = Vec::with_capacity(values.len());
    for (index, value) in values.into_iter().enumerate() {
        let index_name = crate::webidl::string(scope, &index.to_string())?;
        let _ = list.define_own_property(
            scope,
            index_name.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY,
        );
        stored.push(v8::Global::new(scope, value));
    }
    scope
        .get_slot_mut::<TouchListStore>()
        .ok_or_else(|| "TouchList state was not prepared".to_owned())?
        .records
        .insert(list.get_identity_hash().get(), stored);
    Ok(list)
}

pub(crate) fn from_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let mut touches = Vec::new();
    if !value.is_null() && !value.is_undefined() {
        let object = v8::Local::<v8::Object>::try_from(value)
            .map_err(|_| "touch sequence must be an object".to_owned())?;
        let length_key = crate::webidl::string(scope, "length")?;
        let length = object
            .get(scope, length_key.into())
            .and_then(|value| value.uint32_value(scope))
            .unwrap_or(0);
        for index in 0..length {
            let Some(value) = object.get_index(scope, index) else {
                continue;
            };
            let touch = v8::Local::<v8::Object>::try_from(value)
                .map_err(|_| "touch sequence contains a non-object".to_owned())?;
            if !super::touch::is_instance(scope, touch) {
                return Err("touch sequence contains a non-Touch object".to_owned());
            }
            touches.push(touch);
        }
    }
    create(scope, touches)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TouchList': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<TouchListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(value) = record.get(index) {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
