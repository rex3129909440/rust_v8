#[derive(Default)]
pub(crate) struct StylePropertyMapStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StylePropertyMapStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StylePropertyMap", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StylePropertyMapStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StylePropertyMap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_method(scope, p, "append", 1, append)?;
    crate::webidl::define_method(scope, p, "clear", 0, clear)?;
    crate::webidl::define_method(scope, p, "delete", 1, delete)?;
    crate::webidl::define_method(scope, p, "set", 1, set)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::style_property_map_read_only::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StylePropertyMapStore>()
        .ok_or_else(|| "StylePropertyMap state was not prepared".to_owned())?
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
        return Err("cannot create StylePropertyMap".to_owned());
    }
    super::style_property_map_read_only::attach(scope, o);
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'StylePropertyMap': Illegal constructor",
    );
}
fn name(scope: &v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>) -> String {
    crate::webidl::value_to_string(scope, v)
        .trim()
        .to_ascii_lowercase()
}
fn collect(
    scope: &v8::PinScope<'_, '_>,
    a: &v8::FunctionCallbackArguments<'_>,
) -> Vec<v8::Global<v8::Value>> {
    let mut values = Vec::new();
    for i in 1..a.length() {
        values.push(v8::Global::new(scope, a.get(i)));
    }
    values
}
fn append(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::style_property_map_read_only::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if a.length() < 2 {
        crate::webidl::throw_type_error(scope, "append requires a value");
        return;
    }
    let key = name(scope, a.get(0));
    let values = collect(scope, &a);
    super::style_property_map_read_only::update(scope, a.this(), |r| {
        if !r.values.contains_key(&key) {
            r.order.push(key.clone());
        }
        r.values.entry(key).or_default().extend(values);
    });
}
fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::style_property_map_read_only::update(scope, a.this(), |r| {
        r.order.clear();
        r.values.clear();
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::style_property_map_read_only::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let key = name(scope, a.get(0));
    super::style_property_map_read_only::update(scope, a.this(), |r| {
        r.values.remove(&key);
        r.order.retain(|v| v != &key);
    });
}
fn set(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::style_property_map_read_only::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if a.length() < 2 {
        crate::webidl::throw_type_error(scope, "set requires a value");
        return;
    }
    let key = name(scope, a.get(0));
    let values = collect(scope, &a);
    super::style_property_map_read_only::update(scope, a.this(), |r| {
        if !r.values.contains_key(&key) {
            r.order.push(key.clone());
        }
        r.values.insert(key, values);
    });
}
