use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct EventCountsStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Vec<(String, u64)>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EventCountsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "EventCounts", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<EventCountsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "EventCounts",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::event_counts_size_property::define(scope, prototype)?;
    super::event_counts_entries::define(scope, prototype)?;
    super::event_counts_for_each::define(scope, prototype)?;
    super::event_counts_get::define(scope, prototype)?;
    super::event_counts_has::define(scope, prototype)?;
    super::event_counts_keys::define(scope, prototype)?;
    super::event_counts_values::define(scope, prototype)?;
    let entries_key = crate::webidl::string(scope, "entries")?;
    let entries_method = prototype
        .get(scope, entries_key.into())
        .ok_or_else(|| "cannot read EventCounts entries".to_owned())?;
    if prototype.define_own_property(
        scope,
        v8::Symbol::get_iterator(scope).into(),
        entries_method,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define EventCounts iterator".to_owned());
    }
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::move_iterator_to_end(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<EventCountsStore>()
        .ok_or_else(|| "EventCounts state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: Vec<(String, u64)>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create EventCounts".to_owned());
    }
    scope
        .get_slot_mut::<EventCountsStore>()
        .ok_or_else(|| "EventCounts state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), values);
    Ok(object)
}

pub(crate) fn edge_150_initial_values() -> Vec<(String, u64)> {
    [
        "pointerdown",
        "touchend",
        "input",
        "keydown",
        "mouseleave",
        "mouseenter",
        "drop",
        "beforeinput",
        "pointerenter",
        "dragend",
        "pointercancel",
        "compositionupdate",
        "mousedown",
        "dragleave",
        "dragover",
        "mouseup",
        "pointerover",
        "lostpointercapture",
        "mouseover",
        "gotpointercapture",
        "dblclick",
        "keyup",
        "keypress",
        "pointerup",
        "compositionstart",
        "auxclick",
        "dragstart",
        "touchstart",
        "compositionend",
        "pointerout",
        "dragenter",
        "touchcancel",
        "click",
        "contextmenu",
        "mouseout",
        "pointerleave",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), 0))
    .collect()
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, u64)>> {
    scope
        .get_slot::<EventCountsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(values) = snapshot(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, values.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(s, a.get(0));
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some((_, value)) = values.iter().find(|(key, _)| key == &name) {
        r.set(v8::Number::new(s, *value as f64).into())
    } else {
        r.set(v8::undefined(s).into())
    }
}
pub(crate) fn has(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(s, a.get(0));
    if let Some(values) = snapshot(s, a.this()) {
        r.set(v8::Boolean::new(s, values.iter().any(|(key, _)| key == &name)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn iterator(
    s: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(s, "values") else {
        return;
    };
    let Some(method) = array
        .get(s, key.into())
        .and_then(|v| v8::Local::<v8::Function>::try_from(v).ok())
    else {
        return;
    };
    if let Some(value) = method.call(s, array.into(), &[]) {
        r.set(value)
    }
}
pub(crate) fn entries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, values.len() as i32);
    for (index, (key, value)) in values.iter().enumerate() {
        let pair = v8::Array::new(s, 2);
        if let Some(key) = v8::String::new(s, key) {
            let _ = pair.set_index(s, 0, key.into());
        }
        let _ = pair.set_index(s, 1, v8::Number::new(s, *value as f64).into());
        let _ = array.set_index(s, index as u32, pair.into());
    }
    iterator(s, array, r)
}
pub(crate) fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, values.len() as i32);
    for (index, (key, _)) in values.iter().enumerate() {
        if let Some(key) = v8::String::new(s, key) {
            let _ = array.set_index(s, index as u32, key.into());
        }
    }
    iterator(s, array, r)
}
pub(crate) fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, values.len() as i32);
    for (index, (_, value)) in values.iter().enumerate() {
        let _ = array.set_index(s, index as u32, v8::Number::new(s, *value as f64).into());
    }
    iterator(s, array, r)
}
pub(crate) fn for_each(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "The callback must be a function");
        return;
    };
    let Some(values) = snapshot(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    for (key, value) in values {
        let Some(key) = v8::String::new(s, &key) else {
            continue;
        };
        let _ = callback.call(
            s,
            a.get(1),
            &[
                v8::Number::new(s, value as f64).into(),
                key.into(),
                a.this().into(),
            ],
        );
    }
}
