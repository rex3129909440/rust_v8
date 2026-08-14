use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssSkewXStore {
    constructor: crate::webidl::RealmConstructor,
    angles: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssSkewXStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSSkewX", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CssSkewXStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSSkewX",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "ax", get_ax, set_ax)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssSkewXStore>()
        .ok_or_else(|| "CSSSkewX state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "CSSSkewX requires an angle");
        return None;
    }
    Some(v8::Global::new(scope, object))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CSSSkewX requires an angle");
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSSkewX': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let valid_angle = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|object| super::css_numeric_value::is_numeric(scope, object));
    if !valid_angle {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSSkewX': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let Some(angle) = angle(scope, arguments.get(0)) else {
        return;
    };
    scope
        .get_slot_mut::<CssSkewXStore>()
        .expect("CSSSkewX state")
        .angles
        .insert(arguments.this().get_identity_hash().get(), angle);
    super::css_transform_component::attach(scope, arguments.this(), true);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<CssSkewXStore>()?
        .angles
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_ax(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(angle) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &angle).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_ax(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(angle) = angle(scope, arguments.get(0)) else {
        return;
    };
    if let Some(current) = scope.get_slot_mut::<CssSkewXStore>().and_then(|store| {
        store
            .angles
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        *current = angle;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
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
    let angle = record(scope, object)?;
    let angle = super::css_unit_value::record(scope, v8::Local::new(scope, &angle))?;
    Some(format!(
        "skewX({})",
        super::css_unit_value::serialize(&angle)
    ))
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let angle = record(scope, object)?;
    let angle = super::css_unit_value::record(scope, v8::Local::new(scope, &angle))?;
    let mut matrix = super::dom_matrix::identity();
    matrix[4] = degrees(&angle).to_radians().tan();
    Some(matrix)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CssSkewXStore>() {
        store.constructor.remove(realm_id);
    }
}
