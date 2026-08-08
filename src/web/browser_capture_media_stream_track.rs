use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct BrowserCaptureMediaStreamTrackStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BrowserCaptureMediaStreamTrackStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BrowserCaptureMediaStreamTrack", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BrowserCaptureMediaStreamTrackStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BrowserCaptureMediaStreamTrack",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "cropTo", 1, crop_to)?;
    crate::webidl::define_method(scope, prototype, "restrictTo", 1, restrict_to)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::media_stream_track::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BrowserCaptureMediaStreamTrackStore>()
        .ok_or_else(|| "BrowserCaptureMediaStreamTrack state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn resolved(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = scope
        .get_slot::<BrowserCaptureMediaStreamTrackStore>()
        .is_some_and(|store| {
            store
                .objects
                .contains(&arguments.this().get_identity_hash().get())
        });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(resolver) = v8::PromiseResolver::new(scope) {
        let undefined = v8::undefined(scope);
        let _ = resolver.resolve(scope, undefined.into());
        result.set(resolver.get_promise(scope).into());
    }
}

fn crop_to(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    resolved(s, a, r);
}
fn restrict_to(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    resolved(s, a, r);
}
