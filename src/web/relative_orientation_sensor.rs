#[derive(Default)]
pub(crate) struct RelativeOrientationSensorStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(RelativeOrientationSensorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "RelativeOrientationSensor", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<RelativeOrientationSensorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "RelativeOrientationSensor",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = super::orientation_sensor::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, p)?;
    let prototype = crate::webidl::prototype(s, c)?;
    crate::webidl::define_to_string_tag(s, prototype, "RelativeOrientationSensor")?;
    let g = v8::Global::new(s, c);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<RelativeOrientationSensorStore>()
        .ok_or_else(|| "RelativeOrientationSensor state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "Please use the 'new' operator");
        return;
    }
    super::orientation_sensor::attach(
        s,
        a.this(),
        super::orientation_sensor::OrientationKind::Relative,
    );
    r.set(a.this().into())
}
