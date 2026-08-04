use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct ViewportStore {
    constructor: crate::webidl::RealmConstructor,
    native_objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ViewportStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Viewport", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ViewportStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Viewport",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "segments", get_segments)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ViewportStore>()
        .ok_or_else(|| "Viewport state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let viewport = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, viewport, prototype.into()) != Some(true) {
        return Err("cannot create Viewport".to_owned());
    }
    scope
        .get_slot_mut::<ViewportStore>()
        .ok_or_else(|| "Viewport state is unavailable".to_owned())?
        .native_objects
        .insert(viewport.get_identity_hash().get());
    Ok(viewport)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Viewport': Illegal constructor");
}

fn get_segments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let is_native = scope.get_slot::<ViewportStore>().is_some_and(|store| {
        store
            .native_objects
            .contains(&arguments.this().get_identity_hash().get())
    });
    if !is_native {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let rect = super::dom_rect_read_only::RectRecord {
        x: 0.0,
        y: 0.0,
        width: super::window_view_state::inner_width(scope),
        height: super::window_view_state::inner_height(scope),
    };
    let Ok(rect) = super::dom_rect::create(scope, rect) else {
        return;
    };
    let segments = v8::Array::new(scope, 1);
    if segments.set_index(scope, 0, rect.into()) == Some(true) {
        result.set(segments.into());
    }
}
