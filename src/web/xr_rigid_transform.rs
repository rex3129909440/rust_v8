use std::collections::HashMap;
#[derive(Clone)]
struct TransformData {
    position: v8::Global<v8::Object>,
    orientation: v8::Global<v8::Object>,
    matrix: v8::Global<v8::Array>,
}
#[derive(Default)]
pub(crate) struct XrRigidTransformStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TransformData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrRigidTransformStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRRigidTransform", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrRigidTransformStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRRigidTransform",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "position", position)?;
    crate::webidl::define_readonly_accessor(s, p, "orientation", orientation)?;
    crate::webidl::define_readonly_accessor(s, p, "matrix", matrix)?;
    crate::webidl::define_readonly_accessor(s, p, "inverse", inverse)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrRigidTransformStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn object<'s>(s: &mut v8::PinScope<'s, '_>, w: f64) -> v8::Local<'s, v8::Object> {
    let o = v8::Object::new(s);
    let x = v8::String::new(s, "x").unwrap();
    let y = v8::String::new(s, "y").unwrap();
    let z = v8::String::new(s, "z").unwrap();
    let w_key = v8::String::new(s, "w").unwrap();
    let zero = v8::Number::new(s, 0.0);
    let w_value = v8::Number::new(s, w);
    let _ = o.set(s, x.into(), zero.into());
    let _ = o.set(s, y.into(), zero.into());
    let _ = o.set(s, z.into(), zero.into());
    let _ = o.set(s, w_key.into(), w_value.into());
    o
}
fn attach(s: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) {
    let position_object = object(s, 1.0);
    let position = v8::Global::new(s, position_object);
    let orientation_object = object(s, 1.0);
    let orientation = v8::Global::new(s, orientation_object);
    let matrix_array = v8::Array::new(s, 16);
    for i in 0..16 {
        let _ = matrix_array.set_index(
            s,
            i,
            v8::Number::new(s, if i % 5 == 0 { 1.0 } else { 0.0 }).into(),
        );
    }
    let matrix = v8::Global::new(s, matrix_array);
    s.get_slot_mut::<XrRigidTransformStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            TransformData {
                position,
                orientation,
                matrix,
            },
        );
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "use new");
        return;
    }
    attach(s, a.this());
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create XRRigidTransform".to_owned());
    }
    attach(s, o);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<TransformData> {
    s.get_slot::<XrRigidTransformStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn position(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.position).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn orientation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.orientation).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn matrix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.matrix).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn inverse(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(a.this().into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
