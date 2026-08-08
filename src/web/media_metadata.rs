use std::collections::HashMap;

#[derive(Clone)]
struct ArtworkRecord {
    src: String,
    sizes: String,
    media_type: String,
}

#[derive(Clone)]
struct ChapterRecord {
    title: String,
    start_time: f64,
    artwork: Vec<ArtworkRecord>,
}

#[derive(Clone)]
struct MediaMetadataRecord {
    title: String,
    artist: String,
    album: String,
    artwork: v8::Global<v8::Array>,
    chapters: Vec<ChapterRecord>,
}

#[derive(Default)]
pub(crate) struct MediaMetadataStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MediaMetadataRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaMetadataStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaMetadata", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<MediaMetadataStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaMetadata",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "title", get_title, set_title)?;
    crate::webidl::define_accessor(scope, prototype, "artist", get_artist, set_artist)?;
    crate::webidl::define_accessor(scope, prototype, "album", get_album, set_album)?;
    crate::webidl::define_accessor(scope, prototype, "artwork", get_artwork, set_artwork)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "chapterInfo", get_chapter_info)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaMetadataStore>()
        .ok_or_else(|| "MediaMetadata state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MediaMetadataRecord> {
    scope
        .get_slot::<MediaMetadataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut MediaMetadataRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<MediaMetadataStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    operation(record);
    true
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaMetadata': Please use the 'new' operator",
        );
        return;
    }
    let init = arguments.get(0);
    let init = if init.is_undefined() {
        None
    } else {
        let Ok(init) = v8::Local::<v8::Object>::try_from(init) else {
            crate::webidl::throw_type_error(scope, "MediaMetadata init must be an object");
            return;
        };
        Some(init)
    };
    let title = init
        .and_then(|init| string_property(scope, init, "title"))
        .unwrap_or_default();
    let artist = init
        .and_then(|init| string_property(scope, init, "artist"))
        .unwrap_or_default();
    let album = init
        .and_then(|init| string_property(scope, init, "album"))
        .unwrap_or_default();
    let artwork_records = init
        .and_then(|init| object_property(scope, init, "artwork"))
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
        .map(|array| normalize_artwork(scope, array))
        .unwrap_or_default();
    let chapters = init
        .and_then(|init| object_property(scope, init, "chapterInfo"))
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
        .map(|array| normalize_chapters(scope, array))
        .unwrap_or_default();
    let Ok(artwork) = create_artwork_array(scope, &artwork_records) else {
        crate::webidl::throw_type_error(scope, "Cannot create MediaMetadata artwork");
        return;
    };
    let object = arguments.this();
    let metadata = MediaMetadataRecord {
        title,
        artist,
        album,
        artwork: v8::Global::new(scope, artwork),
        chapters,
    };
    scope
        .get_slot_mut::<MediaMetadataStore>()
        .expect("MediaMetadata state")
        .records
        .insert(object.get_identity_hash().get(), metadata);
    result.set(object.into());
}

fn get_title(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_string(scope, arguments, result, |record| &record.title);
}

fn set_title(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(scope, arguments, |record, value| record.title = value);
}

fn get_artist(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_string(scope, arguments, result, |record| &record.artist);
}

fn set_artist(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(scope, arguments, |record, value| record.artist = value);
}

fn get_album(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_string(scope, arguments, result, |record| &record.album);
}

fn set_album(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(scope, arguments, |record, value| record.album = value);
}

fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MediaMetadataRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    assign: impl FnOnce(&mut MediaMetadataRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !update(scope, arguments.this(), |record| assign(record, value)) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_artwork(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, record.artwork).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_artwork(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let records = if arguments.get(0).is_null() || arguments.get(0).is_undefined() {
        Vec::new()
    } else {
        let Ok(array) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "artwork must be an array");
            return;
        };
        normalize_artwork(scope, array)
    };
    let Ok(artwork) = create_artwork_array(scope, &records) else {
        return;
    };
    let artwork = v8::Global::new(scope, artwork);
    if !update(scope, arguments.this(), |record| record.artwork = artwork) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_chapter_info(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let chapters = v8::Array::new(scope, 0);
    for (index, chapter) in record.chapters.iter().enumerate() {
        let Ok(artwork) = create_artwork_array(scope, &chapter.artwork) else {
            return;
        };
        let Ok(value) = super::chapter_information::create(
            scope,
            chapter.title.clone(),
            chapter.start_time,
            artwork,
        ) else {
            return;
        };
        let _ = chapters.set_index(scope, index as u32, value.into());
    }
    result.set(chapters.into());
}

fn normalize_artwork(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
) -> Vec<ArtworkRecord> {
    let mut records = Vec::new();
    for index in 0..array.length() {
        let Some(value) = array.get_index(scope, index) else {
            continue;
        };
        let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
            continue;
        };
        records.push(ArtworkRecord {
            src: string_property(scope, object, "src").unwrap_or_default(),
            sizes: string_property(scope, object, "sizes").unwrap_or_default(),
            media_type: string_property(scope, object, "type").unwrap_or_default(),
        });
    }
    records
}

fn normalize_chapters(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
) -> Vec<ChapterRecord> {
    let mut records = Vec::new();
    for index in 0..array.length() {
        let Some(value) = array.get_index(scope, index) else {
            continue;
        };
        let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
            continue;
        };
        let artwork = object_property(scope, object, "artwork")
            .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
            .map(|array| normalize_artwork(scope, array))
            .unwrap_or_default();
        records.push(ChapterRecord {
            title: string_property(scope, object, "title").unwrap_or_default(),
            start_time: number_property(scope, object, "startTime").unwrap_or(0.0),
            artwork,
        });
    }
    records
}

fn create_artwork_array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    records: &[ArtworkRecord],
) -> Result<v8::Local<'s, v8::Array>, String> {
    let array = v8::Array::new(scope, 0);
    for (index, record) in records.iter().enumerate() {
        let object = v8::Object::new(scope);
        define_string(scope, object, "src", &record.src)?;
        define_string(scope, object, "sizes", &record.sizes)?;
        define_string(scope, object, "type", &record.media_type)?;
        if array.set_index(scope, index as u32, object.into()) != Some(true) {
            return Err("cannot append MediaMetadata artwork".to_owned());
        }
    }
    Ok(array)
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let value = crate::webidl::string(scope, value)?;
    if object.create_data_property(scope, key.into(), value.into()) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define artwork.{name}"))
    }
}

fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn string_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let value = object_property(scope, object, name)?;
    (!value.is_null() && !value.is_undefined())
        .then(|| crate::webidl::value_to_string(scope, value))
}

fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    object_property(scope, object, name)?.number_value(scope)
}
