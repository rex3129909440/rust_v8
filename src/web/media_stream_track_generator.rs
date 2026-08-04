use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamTrackGeneratorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, GeneratorRecord>,
}

#[derive(Clone)]
struct GeneratorRecord {
    writable: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamTrackGeneratorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamTrackGenerator", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamTrackGeneratorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamTrackGenerator",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "writable", get_writable)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::media_stream_track::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamTrackGeneratorStore>()
        .ok_or_else(|| "MediaStreamTrackGenerator state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackGenerator': 1 argument required, but only 0 present.",
        );
        return;
    }
    let kind = generator_kind(scope, arguments.get(0));
    if kind != "audio" && kind != "video" {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackGenerator': Invalid track generator kind",
        );
        return;
    }
    if let Err(message) = super::media_stream_track::attach(scope, arguments.this(), &kind, None) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    let writable = match super::writable_stream::create_empty(scope) {
        Ok(writable) => writable,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let writable = v8::Global::new(scope, writable);
    scope
        .get_slot_mut::<MediaStreamTrackGeneratorStore>()
        .expect("MediaStreamTrackGenerator state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            GeneratorRecord { writable },
        );
    result.set(arguments.this().into());
}

fn generator_kind(scope: &mut v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        if let Some(key) = v8::String::new(scope, "kind") {
            if let Some(kind) = object.get(scope, key.into()) {
                return crate::webidl::value_to_string(scope, kind);
            }
        }
    }
    crate::webidl::value_to_string(scope, value)
}

fn get_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot::<MediaStreamTrackGeneratorStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, &record.writable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
