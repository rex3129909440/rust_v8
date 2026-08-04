use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "mediaText",
        get_media_text,
        set_media_text,
    )?;
    crate::webidl::define_method(scope, prototype, "appendMedium", 1, append_medium)?;
    crate::webidl::define_method(scope, prototype, "deleteMedium", 1, delete_medium)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaListStore>()
        .ok_or_else(|| "MediaList state was not prepared".to_owned())?
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
        "Failed to construct 'MediaList': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    media_text: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaList".to_owned());
    }
    scope
        .get_slot_mut::<MediaListStore>()
        .ok_or_else(|| "MediaList state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), parse_media(media_text));
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Vec<String>> {
    scope
        .get_slot::<MediaListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    Some(record(scope, object)?.join(", "))
}

pub(crate) fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    media_text: &str,
) -> bool {
    if let Some(media) = scope
        .get_slot_mut::<MediaListStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        *media = parse_media(media_text);
        true
    } else {
        false
    }
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(media) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, media.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_media_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(media) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &media.join(", ")) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_media_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_media_text(s, a, r);
}
fn to_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_media_text(s, a, r);
}

fn set_media_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let media = parse_media(&crate::webidl::value_to_string(scope, arguments.get(0)));
    if let Some(stored) = scope.get_slot_mut::<MediaListStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        *stored = media;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn append_medium(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let medium = crate::webidl::value_to_string(scope, arguments.get(0))
        .trim()
        .to_owned();
    if let Some(media) = scope.get_slot_mut::<MediaListStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        if !medium.is_empty() && !media.contains(&medium) {
            media.push(medium);
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn delete_medium(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let medium = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(media) = scope.get_slot_mut::<MediaListStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let length = media.len();
    media.retain(|existing| existing != &medium);
    if length == media.len() {
        match super::dom_exception::create(
            scope,
            "The medium was not found.".to_owned(),
            "NotFoundError".to_owned(),
        ) {
            Ok(exception) => {
                scope.throw_exception(exception.into());
            }
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(0) as usize;
    let Some(media) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(medium) = media.get(index) {
        if let Some(value) = v8::String::new(scope, medium) {
            result.set(value.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}

fn parse_media(media_text: &str) -> Vec<String> {
    media_text
        .split(',')
        .map(str::trim)
        .filter(|medium| !medium.is_empty())
        .map(str::to_owned)
        .collect()
}
