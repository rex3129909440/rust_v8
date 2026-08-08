use std::collections::HashMap;

#[derive(Clone)]
struct ChapterInformationRecord {
    title: String,
    start_time: f64,
    artwork: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct ChapterInformationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ChapterInformationRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ChapterInformationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ChapterInformation", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ChapterInformationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ChapterInformation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "title", get_title)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "startTime", get_start_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "artwork", get_artwork)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ChapterInformationStore>()
        .ok_or_else(|| "ChapterInformation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    title: String,
    start_time: f64,
    artwork: v8::Local<'_, v8::Array>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let chapter = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, chapter, prototype.into()) != Some(true) {
        return Err("cannot create ChapterInformation".to_owned());
    }
    let record = ChapterInformationRecord {
        title,
        start_time,
        artwork: v8::Global::new(scope, artwork),
    };
    scope
        .get_slot_mut::<ChapterInformationStore>()
        .ok_or_else(|| "ChapterInformation state was not prepared".to_owned())?
        .records
        .insert(chapter.get_identity_hash().get(), record);
    Ok(chapter)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ChapterInformationRecord> {
    scope
        .get_slot::<ChapterInformationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ChapterInformation': Illegal constructor",
    );
}

fn get_title(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.title) {
        result.set(value.into());
    }
}

fn get_start_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.start_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_artwork(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.artwork).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
