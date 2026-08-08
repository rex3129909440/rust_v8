#[derive(Default)]
pub(crate) struct GpuExternalTextureStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuExternalTextureStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUExternalTexture", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(x) = scope
        .get_slot::<GpuExternalTextureStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &x));
    }
    let c = crate::webidl::create_function(
        scope,
        "GPUExternalTexture",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "label", get_label, set_label)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<GpuExternalTextureStore>()
        .ok_or_else(|| "GPUExternalTexture state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "GPUExternalTexture is not directly constructible")
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    label: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, constructor)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create GPUExternalTexture".to_owned());
    }
    super::gpu_label_support::attach(scope, o, label);
    Ok(o)
}
fn get_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = super::gpu_label_support::get(s, a.this())
        && let Some(v) = v8::String::new(s, &x)
    {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if !super::gpu_label_support::set(s, a.this(), v) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuExternalTextureStore>() {
        store.constructor.remove(realm_id);
    }
}
