use std::collections::HashMap;

#[derive(Clone)]
struct QuadRecord {
    p1: v8::Global<v8::Object>,
    p2: v8::Global<v8::Object>,
    p3: v8::Global<v8::Object>,
    p4: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct DomQuadStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, QuadRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomQuadStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMQuad", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<DomQuadStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMQuad",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "p1", get_p1)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "p2", get_p2)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "p3", get_p3)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "p4", get_p4)?;
    crate::webidl::define_method(scope, prototype, "getBounds", 0, get_bounds)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "fromQuad", 0, from_quad)?;
    crate::webidl::define_method(scope, constructor.into(), "fromRect", 0, from_rect)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomQuadStore>()
        .ok_or_else(|| "DOMQuad state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    points: [super::dom_point_read_only::PointRecord; 4],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DOMQuad".to_owned());
    }
    attach(scope, object, points)?;
    Ok(object)
}

fn quad_points_from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> [super::dom_point_read_only::PointRecord; 4] {
    let default = super::dom_point_read_only::PointRecord {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return [default; 4];
    };
    let point = |name: &str| {
        v8::String::new(scope, name)
            .and_then(|key| object.get(scope, key.into()))
            .map(|value| super::dom_point_read_only::from_value(scope, value))
            .unwrap_or(default)
    };
    [point("p1"), point("p2"), point("p3"), point("p4")]
}

fn from_quad(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let points = quad_points_from_value(scope, arguments.get(0));
    match create(scope, points) {
        Ok(quad) => result.set(quad.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn from_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let rect = super::dom_rect_read_only::from_value(scope, arguments.get(0));
    let point = |x, y| super::dom_point_read_only::PointRecord {
        x,
        y,
        z: 0.0,
        w: 1.0,
    };
    let points = [
        point(rect.x, rect.y),
        point(rect.x + rect.width, rect.y),
        point(rect.x + rect.width, rect.y + rect.height),
        point(rect.x, rect.y + rect.height),
    ];
    match create(scope, points) {
        Ok(quad) => result.set(quad.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "DOMQuad must be constructed with new");
        return;
    }
    let p1 = super::dom_point_read_only::from_value(scope, arguments.get(0));
    let p2 = super::dom_point_read_only::from_value(scope, arguments.get(1));
    let p3 = super::dom_point_read_only::from_value(scope, arguments.get(2));
    let p4 = super::dom_point_read_only::from_value(scope, arguments.get(3));
    match attach(scope, arguments.this(), [p1, p2, p3, p4]) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    points: [super::dom_point_read_only::PointRecord; 4],
) -> Result<(), String> {
    let p1 = super::dom_point::create(scope, points[0])?;
    let p2 = super::dom_point::create(scope, points[1])?;
    let p3 = super::dom_point::create(scope, points[2])?;
    let p4 = super::dom_point::create(scope, points[3])?;
    let record = QuadRecord {
        p1: v8::Global::new(scope, p1),
        p2: v8::Global::new(scope, p2),
        p3: v8::Global::new(scope, p3),
        p4: v8::Global::new(scope, p4),
    };
    scope
        .get_slot_mut::<DomQuadStore>()
        .ok_or_else(|| "DOMQuad state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(())
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<QuadRecord> {
    scope
        .get_slot::<DomQuadStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(QuadRecord) -> v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_p1(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_point(s, a, r, |v| v.p1)
}
fn get_p2(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_point(s, a, r, |v| v.p2)
}
fn get_p3(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_point(s, a, r, |v| v.p3)
}
fn get_p4(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_point(s, a, r, |v| v.p4)
}

fn point_values(
    scope: &v8::PinScope<'_, '_>,
    record: &QuadRecord,
) -> Option<[super::dom_point_read_only::PointRecord; 4]> {
    Some([
        super::dom_point_read_only::record(scope, v8::Local::new(scope, &record.p1))?,
        super::dom_point_read_only::record(scope, v8::Local::new(scope, &record.p2))?,
        super::dom_point_read_only::record(scope, v8::Local::new(scope, &record.p3))?,
        super::dom_point_read_only::record(scope, v8::Local::new(scope, &record.p4))?,
    ])
}

fn get_bounds(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(points) = point_values(scope, &record) else {
        crate::webidl::throw_type_error(scope, "Invalid DOMQuad");
        return;
    };
    let left = points
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let top = points
        .iter()
        .map(|point| point.y)
        .fold(f64::INFINITY, f64::min);
    let right = points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let bottom = points
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);
    match super::dom_rect::create(
        scope,
        super::dom_rect_read_only::RectRecord {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        },
    ) {
        Ok(rect) => result.set(rect.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
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
    let object = v8::Object::new(scope);
    define_point(scope, object, "p1", v8::Local::new(scope, &record.p1));
    define_point(scope, object, "p2", v8::Local::new(scope, &record.p2));
    define_point(scope, object, "p3", v8::Local::new(scope, &record.p3));
    define_point(scope, object, "p4", v8::Local::new(scope, &record.p4));
    result.set(object.into());
}

fn define_point(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    point: v8::Local<'_, v8::Object>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), point.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomQuadStore>() {
        store.constructor.remove(realm_id);
    }
}
