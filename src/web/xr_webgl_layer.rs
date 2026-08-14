use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrWebGlLayerStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrWebGlLayerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRWebGLLayer", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrWebGlLayerStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRWebGLLayer",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "antialias", true_value)?;
    crate::webidl::define_readonly_accessor(s, p, "ignoreDepthValues", false_value)?;
    crate::webidl::define_readonly_accessor(s, p, "framebufferWidth", width)?;
    crate::webidl::define_readonly_accessor(s, p, "framebufferHeight", height)?;
    crate::webidl::define_readonly_accessor(s, p, "framebuffer", null)?;
    crate::webidl::define_method(s, p, "getViewport", 1, viewport)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_method(
        s,
        c.into(),
        "getNativeFramebufferScaleFactor",
        1,
        get_native_framebuffer_scale_factor,
    )?;
    let parent = super::xr_layer::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrWebGlLayerStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn get_native_framebuffer_scale_factor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'getNativeFramebufferScaleFactor' on 'XRWebGLLayer': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(session) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'getNativeFramebufferScaleFactor' on 'XRWebGLLayer': parameter 1 is not of type 'XRSession'.",
        );
        return;
    };
    let Some(scale) = super::xr_session::native_framebuffer_scale_factor(s, session) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'getNativeFramebufferScaleFactor' on 'XRWebGLLayer': parameter 1 is not of type 'XRSession'.",
        );
        return;
    };
    r.set(v8::Number::new(s, scale).into());
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(s, "2 arguments required");
        return;
    }
    if a.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'XRWebGLLayer': parameter 1 is not of type 'XRSession'.",
        );
        return;
    }
    let valid_session = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .is_some_and(|object| {
            super::structured_clone::inherits_platform_interface(s, object, "XRSession")
        });
    if !valid_session {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'XRWebGLLayer': parameter 1 is not of type 'XRSession'.",
        );
        return;
    }
    super::xr_layer::attach(s, a.this());
    s.get_slot_mut::<XrWebGlLayerStore>()
        .expect("XRWebGLLayer state")
        .instances
        .insert(a.this().get_identity_hash().get());
    r.set(a.this().into())
}
fn require(s: &mut v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> bool {
    let valid = s.get_slot::<XrWebGlLayerStore>().is_some_and(|store| {
        store
            .instances
            .contains(&a.this().get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    valid
}
fn true_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Boolean::new(s, true).into())
}
fn false_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Boolean::new(s, false).into())
}
fn width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Integer::new(s, 1280).into())
}
fn height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::Integer::new(s, 720).into())
}
fn null(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::null(s).into())
}
fn viewport(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    if let Ok(v) = super::xr_viewport::create(s) {
        r.set(v.into())
    }
}
