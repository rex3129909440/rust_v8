use std::collections::HashSet;
#[derive(Default)]
pub(crate) struct GyroscopeStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GyroscopeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Gyroscope", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<GyroscopeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Gyroscope",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "x", get_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "y", get_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "z", get_z)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::sensor::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<GyroscopeStore>()
        .ok_or_else(|| "Gyroscope state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Please use the 'new' operator");
        return;
    }
    super::sensor::attach(scope, arguments.this());
    scope
        .get_slot_mut::<GyroscopeStore>()
        .expect("Gyroscope state")
        .instances
        .insert(arguments.this().get_identity_hash().get());
    result.set(arguments.this().into())
}
fn coordinate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    component: usize,
) {
    if scope.get_slot::<GyroscopeStore>().is_some_and(|store| {
        store
            .instances
            .contains(&arguments.this().get_identity_hash().get())
    }) {
        let value = crate::fingerprint::edge(scope).sensors.gyroscope[component];
        result.set(v8::Number::new(scope, value).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    coordinate(s, a, r, 0)
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    coordinate(s, a, r, 1)
}
fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    coordinate(s, a, r, 2)
}
