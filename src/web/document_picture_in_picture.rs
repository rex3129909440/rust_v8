use std::collections::HashMap;

#[derive(Clone, Default)]
struct DocumentPictureInPictureRecord {
    window: Option<v8::Global<v8::Object>>,
    onenter: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct DocumentPictureInPictureStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DocumentPictureInPictureRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentPictureInPictureStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DocumentPictureInPicture", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DocumentPictureInPictureStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DocumentPictureInPicture",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "window", get_window)?;
    crate::webidl::define_accessor(scope, prototype, "onenter", get_onenter, set_onenter)?;
    crate::webidl::define_method(scope, prototype, "requestWindow", 0, request_window)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DocumentPictureInPictureStore>()
        .ok_or_else(|| "DocumentPictureInPicture state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'DocumentPictureInPicture': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DocumentPictureInPicture".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<DocumentPictureInPictureStore>()
        .ok_or_else(|| "DocumentPictureInPicture state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            DocumentPictureInPictureRecord::default(),
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DocumentPictureInPictureRecord> {
    scope
        .get_slot::<DocumentPictureInPictureStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.window {
        Some(window) => result.set(v8::Local::new(scope, &window).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn get_onenter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| record.onenter);
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn set_onenter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<DocumentPictureInPictureStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.onenter = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn request_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(error) = super::dom_exception::create(
        scope,
        "Document Picture-in-Picture requires transient user activation.".to_owned(),
        "NotAllowedError".to_owned(),
    ) else {
        return;
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, error.into()) {
        result.set(promise.into());
    }
}
