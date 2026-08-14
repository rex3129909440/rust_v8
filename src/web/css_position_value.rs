use std::collections::HashMap;

#[derive(Clone)]
struct CssPositionValueRecord {
    x: v8::Global<v8::Object>,
    y: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssPositionValueStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssPositionValueRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssPositionValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSPositionValue", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssPositionValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSPositionValue",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "x", get_x, set_x)?;
    crate::webidl::define_accessor(scope, prototype, "y", get_y, set_y)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_style_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssPositionValueStore>()
        .ok_or_else(|| "CSSPositionValue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn numeric(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Object>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    if super::css_unit_value::record(scope, object).is_some() {
        Some(v8::Global::new(scope, object))
    } else {
        crate::webidl::throw_type_error(scope, "CSSPositionValue requires numeric values");
        None
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "CSSPositionValue requires x and y");
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSPositionValue': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let valid_x = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|object| super::css_numeric_value::is_numeric(scope, object));
    if !valid_x {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSPositionValue': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let Some(x) = numeric(scope, arguments.get(0)) else {
        return;
    };
    let Some(y) = numeric(scope, arguments.get(1)) else {
        return;
    };
    scope
        .get_slot_mut::<CssPositionValueStore>()
        .expect("CSSPositionValue state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CssPositionValueRecord { x, y },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssPositionValueRecord> {
    scope
        .get_slot::<CssPositionValueStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    x_axis: bool,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, object) {
        let value = if x_axis { record.x } else { record.y };
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    x_axis: bool,
) {
    if record(scope, object).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(value) = numeric(scope, value) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<CssPositionValueStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        if x_axis {
            record.x = value;
        } else {
            record.y = value;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), true, r);
}
fn set_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), a.get(0), true);
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), false, r);
}
fn set_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), a.get(0), false);
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let x = super::css_unit_value::record(scope, v8::Local::new(scope, &record.x))?;
    let y = super::css_unit_value::record(scope, v8::Local::new(scope, &record.y))?;
    Some(format!(
        "{} {}",
        super::css_unit_value::serialize(&x),
        super::css_unit_value::serialize(&y)
    ))
}
