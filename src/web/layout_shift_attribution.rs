use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LayoutShiftAttributionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AttributionRecord>,
}

#[derive(Clone)]
struct AttributionRecord {
    node: Option<v8::Global<v8::Object>>,
    previous_rect: v8::Global<v8::Object>,
    current_rect: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LayoutShiftAttributionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LayoutShiftAttribution", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<LayoutShiftAttributionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "LayoutShiftAttribution",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "node", get_node)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "previousRect", get_previous_rect)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "currentRect", get_current_rect)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<LayoutShiftAttributionStore>()
        .ok_or_else(|| "LayoutShiftAttribution state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'LayoutShiftAttribution': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    node: Option<v8::Local<'s, v8::Object>>,
    previous_rect: v8::Local<'s, v8::Object>,
    current_rect: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create LayoutShiftAttribution".to_owned());
    }
    let node = node.map(|node| v8::Global::new(scope, node));
    let previous_rect = v8::Global::new(scope, previous_rect);
    let current_rect = v8::Global::new(scope, current_rect);
    scope
        .get_slot_mut::<LayoutShiftAttributionStore>()
        .ok_or_else(|| "LayoutShiftAttribution state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AttributionRecord {
                node,
                previous_rect,
                current_rect,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AttributionRecord> {
    scope
        .get_slot::<LayoutShiftAttributionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AttributionRecord) -> Option<&v8::Global<v8::Object>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_node(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| x.node.as_ref());
}
fn get_previous_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| Some(&x.previous_rect));
}
fn get_current_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| Some(&x.current_rect));
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    if let Some(node) = record.node {
        define(scope, output, "node", v8::Local::new(scope, &node).into());
    } else {
        define(scope, output, "node", v8::null(scope).into());
    }
    define(
        scope,
        output,
        "previousRect",
        v8::Local::new(scope, &record.previous_rect).into(),
    );
    define(
        scope,
        output,
        "currentRect",
        v8::Local::new(scope, &record.current_rect).into(),
    );
    result.set(output.into());
}

fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), value);
    }
}
