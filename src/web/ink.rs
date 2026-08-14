#[derive(Default)]
pub(crate) struct InkStore {
    constructor: crate::webidl::RealmConstructor,
    instances: std::collections::HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(InkStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Ink", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<InkStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Ink",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "requestPresenter", 0, request_presenter)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<InkStore>()
        .ok_or_else(|| "Ink state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Ink".to_owned());
    }
    scope
        .get_slot_mut::<InkStore>()
        .ok_or_else(|| "Ink state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Ink': Illegal constructor");
}

fn request_presenter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if scope.get_slot::<InkStore>().is_none_or(|store| {
        !store
            .instances
            .contains(&arguments.this().get_identity_hash().get())
    }) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Ink", "requestPresenter", result);
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let presentation_area = options
        .and_then(|options| {
            let key = v8::String::new(scope, "presentationArea")?;
            options.get(scope, key.into())
        })
        .filter(|value| !value.is_null_or_undefined())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
    if let Some(area) = presentation_area {
        if super::element::record(scope, area).is_none()
            && super::offscreen_canvas::dimensions(scope, area).is_none()
        {
            crate::webidl::throw_type_error(scope, "presentationArea must be an Element");
            return;
        }
    }
    match super::delegated_ink_trail_presenter::create(scope, presentation_area) {
        Ok(presenter) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, presenter.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
