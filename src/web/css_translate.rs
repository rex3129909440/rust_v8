use std::collections::HashMap;

#[derive(Clone)]
struct CssTranslateRecord {
    x: v8::Global<v8::Object>,
    y: v8::Global<v8::Object>,
    z: v8::Global<v8::Object>,
    original_2d: bool,
}

#[derive(Default)]
pub(crate) struct CssTranslateStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssTranslateRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssTranslateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSTranslate", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssTranslateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSTranslate",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "x", get_x, set_x)?;
    crate::webidl::define_accessor(scope, prototype, "y", get_y, set_y)?;
    crate::webidl::define_accessor(scope, prototype, "z", get_z, set_z)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssTranslateStore>()
        .ok_or_else(|| "CSSTranslate state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn unit_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Global<v8::Object>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok();
    if let Some(object) = object
        && super::css_unit_value::record(scope, object).is_some()
    {
        Some(v8::Global::new(scope, object))
    } else {
        crate::webidl::throw_type_error(scope, &format!("{name} must be a CSSUnitValue"));
        None
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "CSSTranslate requires x and y");
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSTranslate': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let valid_x = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|object| super::css_numeric_value::is_numeric(scope, object));
    if !valid_x {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSTranslate': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let Some(x) = unit_value(scope, arguments.get(0), "x") else {
        return;
    };
    let Some(y) = unit_value(scope, arguments.get(1), "y") else {
        return;
    };
    let z = if arguments.get(2).is_undefined() {
        match super::css_unit_value::create(scope, 0.0, "px") {
            Ok(z) => v8::Global::new(scope, z),
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    } else {
        let Some(z) = unit_value(scope, arguments.get(2), "z") else {
            return;
        };
        z
    };
    let is_2d = super::css_unit_value::record(scope, v8::Local::new(scope, &z))
        .is_some_and(|value| value.value == 0.0);
    let record = CssTranslateRecord {
        x,
        y,
        z,
        original_2d: is_2d,
    };
    scope
        .get_slot_mut::<CssTranslateStore>()
        .expect("CSSTranslate state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    super::css_transform_component::attach(scope, arguments.this(), is_2d);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssTranslateRecord> {
    scope
        .get_slot::<CssTranslateStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(CssTranslateRecord) -> v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a, r, |record| record.x);
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a, r, |record| record.y);
}
fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a, r, |record| record.z);
}

fn set_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    component: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(value) = unit_value(scope, arguments.get(0), component) else {
        return;
    };
    let Some(record) = scope.get_slot_mut::<CssTranslateStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match component {
        "x" => record.x = value,
        "y" => record.y = value,
        _ => record.z = value,
    }
}

fn set_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a, "x");
}
fn set_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a, "y");
}
fn set_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a, "z");
}

pub(crate) fn serialize_component(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let Some(x) = super::css_unit_value::record(scope, v8::Local::new(scope, &record.x)) else {
        return None;
    };
    let Some(y) = super::css_unit_value::record(scope, v8::Local::new(scope, &record.y)) else {
        return None;
    };
    let Some(z) = super::css_unit_value::record(scope, v8::Local::new(scope, &record.z)) else {
        return None;
    };
    Some(if record.original_2d {
        format!(
            "translate({}, {})",
            super::css_unit_value::serialize(&x),
            super::css_unit_value::serialize(&y)
        )
    } else {
        format!(
            "translate3d({}, {}, {})",
            super::css_unit_value::serialize(&x),
            super::css_unit_value::serialize(&y),
            super::css_unit_value::serialize(&z)
        )
    })
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let record = record(scope, object)?;
    let x = super::css_unit_value::record(scope, v8::Local::new(scope, &record.x))?;
    let y = super::css_unit_value::record(scope, v8::Local::new(scope, &record.y))?;
    let z = super::css_unit_value::record(scope, v8::Local::new(scope, &record.z))?;
    let mut matrix = super::dom_matrix::identity();
    matrix[12] = x.value;
    matrix[13] = y.value;
    matrix[14] = z.value;
    Some(matrix)
}
