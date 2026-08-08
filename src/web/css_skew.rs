use std::collections::HashMap;

#[derive(Clone)]
struct CssSkewRecord {
    ax: v8::Global<v8::Object>,
    ay: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssSkewStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssSkewRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssSkewStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSSkew", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssSkewStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSSkew",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "ax", get_ax, set_ax)?;
    crate::webidl::define_accessor(scope, prototype, "ay", get_ay, set_ay)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssSkewStore>()
        .ok_or_else(|| "CSSSkew state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn angle(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Object>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let record = super::css_unit_value::record(scope, object)?;
    if !matches!(record.unit.as_str(), "deg" | "rad" | "grad" | "turn") {
        crate::webidl::throw_type_error(scope, "CSSSkew requires angle values");
        return None;
    }
    Some(v8::Global::new(scope, object))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "CSSSkew requires ax and ay");
        return;
    }
    let Some(ax) = angle(scope, arguments.get(0)) else {
        return;
    };
    let Some(ay) = angle(scope, arguments.get(1)) else {
        return;
    };
    scope
        .get_slot_mut::<CssSkewStore>()
        .expect("CSSSkew state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CssSkewRecord { ax, ay },
        );
    super::css_transform_component::attach(scope, arguments.this(), true);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssSkewRecord> {
    scope
        .get_slot::<CssSkewStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_angle(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    select_ax: bool,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, object) {
        let value = if select_ax { record.ax } else { record.ay };
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_angle(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    select_ax: bool,
) {
    let Some(value) = angle(scope, value) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<CssSkewStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        if select_ax {
            record.ax = value;
        } else {
            record.ay = value;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_ax(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_angle(scope, arguments.this(), true, result);
}

fn set_ax(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_angle(scope, arguments.this(), arguments.get(0), true);
}

fn get_ay(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_angle(scope, arguments.this(), false, result);
}

fn set_ay(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_angle(scope, arguments.this(), arguments.get(0), false);
}

fn degrees(record: &super::css_unit_value::CssUnitRecord) -> f64 {
    match record.unit.as_str() {
        "rad" => record.value.to_degrees(),
        "grad" => record.value * 0.9,
        "turn" => record.value * 360.0,
        _ => record.value,
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let ax = super::css_unit_value::record(scope, v8::Local::new(scope, &record.ax))?;
    let ay = super::css_unit_value::record(scope, v8::Local::new(scope, &record.ay))?;
    Some(format!(
        "skew({}, {})",
        super::css_unit_value::serialize(&ax),
        super::css_unit_value::serialize(&ay)
    ))
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let record = record(scope, object)?;
    let ax = super::css_unit_value::record(scope, v8::Local::new(scope, &record.ax))?;
    let ay = super::css_unit_value::record(scope, v8::Local::new(scope, &record.ay))?;
    let mut matrix = super::dom_matrix::identity();
    matrix[4] = degrees(&ax).to_radians().tan();
    matrix[1] = degrees(&ay).to_radians().tan();
    Some(matrix)
}
