use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct FontFaceSetStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}

#[derive(Clone)]
struct Record {
    realm_id: i32,
    faces: Vec<v8::Global<v8::Object>>,
    face_ids: Vec<i32>,
    ready: v8::Global<v8::Promise>,
    handlers: HashMap<&'static str, v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FontFaceSetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FontFaceSet", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FontFaceSetStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FontFaceSet",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "onloading", get_onloading, set_onloading)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onloadingdone",
        get_onloading_done,
        set_onloading_done,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onloadingerror",
        get_onloading_error,
        set_onloading_error,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ready", get_ready)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "status", get_status)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "check", 1, check)?;
    crate::webidl::define_method(scope, prototype, "load", 1, load)?;
    crate::webidl::define_method(scope, prototype, "add", 1, add)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, values)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let values_key = crate::webidl::string(scope, "values")?;
    let values_function = prototype
        .get(scope, values_key.into())
        .ok_or_else(|| "FontFaceSet.values is unavailable".to_owned())?;
    if prototype.define_own_property(
        scope,
        v8::Symbol::get_iterator(scope).into(),
        values_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define FontFaceSet iterator".to_owned());
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    if crate::browser_surface::current_version(scope).major() <= 150 {
        let constructor_key = crate::webidl::string(scope, "constructor")?;
        if prototype.delete(scope, constructor_key.into()) != Some(true) {
            return Err("cannot remove private FontFaceSet prototype constructor".to_owned());
        }
    }
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FontFaceSetStore>()
        .ok_or_else(|| "FontFaceSet state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn resolved<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    super::writable_stream::resolved_promise(scope, value)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    faces: Vec<v8::Global<v8::Object>>,
) -> Result<(), String> {
    for face in &faces {
        let face = v8::Local::new(scope, face);
        if !super::font_face::is_font_face(scope, face) {
            return Err("FontFaceSet requires FontFace entries".to_owned());
        }
    }
    let resolver = v8::PromiseResolver::new(scope)
        .ok_or_else(|| "cannot create FontFaceSet.ready".to_owned())?;
    let promise = resolver.get_promise(scope);
    let _ = resolver.resolve(scope, object.into());
    let ready = v8::Global::new(scope, promise);
    let face_ids = faces
        .iter()
        .map(|face| v8::Local::new(scope, face).get_identity_hash().get())
        .collect();
    let realm_id = crate::webidl::realm_id(scope);
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<FontFaceSetStore>()
        .ok_or_else(|| "FontFaceSet state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            Record {
                realm_id,
                faces,
                face_ids,
                ready,
                handlers: HashMap::new(),
            },
        );
    let registered = scope
        .get_slot::<FontFaceSetStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .map(|record| record.faces.clone())
        .unwrap_or_default();
    for face in registered {
        super::font_face::register_with_shaper(scope, realm_id, v8::Local::new(scope, &face))?;
    }
    Ok(())
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create FontFaceSet".to_owned());
    }
    attach(scope, object, Vec::new())?;
    Ok(object)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "FontFaceSet requires new");
        return;
    }
    let faces = sequence_faces(scope, arguments.get(0)).unwrap_or_default();
    if let Err(message) = attach(scope, arguments.this(), faces) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    result.set(arguments.this().into());
}

fn sequence_faces(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    if value.is_undefined() {
        return Some(Vec::new());
    }
    let sequence = v8::Local::<v8::Object>::try_from(value).ok()?;
    let length = v8::String::new(scope, "length")
        .and_then(|key| sequence.get(scope, key.into()))
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut faces = Vec::new();
    for index in 0..length {
        let face = sequence.get_index(scope, index)?;
        let face = v8::Local::<v8::Object>::try_from(face).ok()?;
        faces.push(v8::Global::new(scope, face));
    }
    Some(faces)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<FontFaceSetStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_status(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else if let Some(value) = v8::String::new(scope, "loaded") {
        result.set(value.into());
    }
}

fn get_ready(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Local::new(scope, &record.ready).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => {
            result.set(v8::Integer::new_from_unsigned(scope, record.faces.len() as u32).into())
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn check(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let font = crate::webidl::value_to_string(scope, arguments.get(0));
    // FontFaceSet.check() reports whether the requested font faces are ready;
    // an unavailable family falls through the CSS font list and therefore does
    // not by itself make the result false in Chromium.
    result.set(v8::Boolean::new(scope, !font.trim().is_empty()).into());
}

fn face_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    faces: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, faces.len() as i32);
    for (index, face) in faces.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, face).into());
    }
    array
}

fn load(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = face_array(scope, &record.faces);
    if let Ok(promise) = resolved(scope, array.into()) {
        result.set(promise.into());
    }
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(face) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "FontFaceSet.add requires a FontFace");
        return;
    };
    if !super::font_face::is_font_face(scope, face) {
        crate::webidl::throw_type_error(scope, "FontFaceSet.add requires a FontFace");
        return;
    }
    let identity = face.get_identity_hash().get();
    let set_identity = arguments.this().get_identity_hash().get();
    if !scope
        .get_slot::<FontFaceSetStore>()
        .is_some_and(|store| store.records.contains_key(&set_identity))
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let already_present = scope
        .get_slot::<FontFaceSetStore>()
        .and_then(|store| store.records.get(&set_identity))
        .is_some_and(|record| record.face_ids.contains(&identity));
    let realm_id = record(scope, arguments.this())
        .map(|record| record.realm_id)
        .unwrap_or_else(|| crate::webidl::realm_id(scope));
    if !already_present
        && let Err(message) = super::font_face::register_with_shaper(scope, realm_id, face)
    {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    let face = v8::Global::new(scope, face);
    let Some(record) = scope
        .get_slot_mut::<FontFaceSetStore>()
        .and_then(|store| store.records.get_mut(&set_identity))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.face_ids.contains(&identity) {
        record.faces.push(face);
        record.face_ids.push(identity);
    }
    result.set(arguments.this().into());
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let (realm_id, removed) = match scope.get_slot_mut::<FontFaceSetStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        Some(record) => {
            let removed = std::mem::take(&mut record.face_ids);
            record.faces.clear();
            (record.realm_id, removed)
        }
        None => {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        }
    };
    for identity in removed {
        super::font_face::unregister_with_shaper(scope, realm_id, identity);
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .map(|face| face.get_identity_hash().get());
    let Some(record) = scope.get_slot_mut::<FontFaceSetStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let realm_id = record.realm_id;
    let removed_identity = identity
        .and_then(|identity| record.face_ids.iter().position(|face| *face == identity))
        .map(|index| {
            let identity = record.face_ids.remove(index);
            record.faces.remove(index);
            identity
        });
    if let Some(identity) = removed_identity {
        super::font_face::unregister_with_shaper(scope, realm_id, identity);
    }
    result.set(v8::Boolean::new(scope, removed_identity.is_some()).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    let removed = scope
        .get_slot_mut::<FontFaceSetStore>()
        .map(|store| {
            let identities = store
                .records
                .iter()
                .filter_map(|(identity, record)| (record.realm_id == realm_id).then_some(*identity))
                .collect::<Vec<_>>();
            identities
                .into_iter()
                .filter_map(|identity| store.records.remove(&identity))
                .flat_map(|record| record.face_ids)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for identity in removed {
        super::font_face::unregister_with_shaper(scope, realm_id, identity);
    }
    crate::font_shaping::cleanup_realm(scope, realm_id);
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .map(|face| face.get_identity_hash().get());
    match record(scope, arguments.this()) {
        Some(record) => result.set(
            v8::Boolean::new(
                scope,
                identity.is_some_and(|identity| record.face_ids.contains(&identity)),
            )
            .into(),
        ),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn iterator_from_array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
) -> Option<v8::Local<'s, v8::Value>> {
    let method = array.get(scope, v8::Symbol::get_iterator(scope).into())?;
    let method = v8::Local::<v8::Function>::try_from(method).ok()?;
    method.call(scope, array.into(), &[])
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = face_array(scope, &record.faces);
    if let Some(iterator) = iterator_from_array(scope, array) {
        result.set(iterator);
    }
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.faces.len() as i32);
    for (index, face) in record.faces.iter().enumerate() {
        let face = v8::Local::new(scope, face);
        let pair = v8::Array::new(scope, 2);
        let _ = pair.set_index(scope, 0, face.into());
        let _ = pair.set_index(scope, 1, face.into());
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    if let Some(iterator) = iterator_from_array(scope, array) {
        result.set(iterator);
    }
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "FontFaceSet.forEach requires a callback");
        return;
    };
    let receiver = if arguments.get(1).is_undefined() {
        v8::undefined(scope).into()
    } else {
        arguments.get(1)
    };
    for face in record.faces {
        let face = v8::Local::new(scope, &face);
        let _ = callback.call(
            scope,
            receiver,
            &[face.into(), face.into(), arguments.this().into()],
        );
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &'static str,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.handlers.get(name) {
            Some(value) => result.set(v8::Local::new(scope, value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &'static str,
) {
    let value = arguments
        .get(0)
        .is_object()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    let present = value.is_some();
    let Some(record) = scope.get_slot_mut::<FontFaceSetStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = value {
        record.handlers.insert(name, value);
    } else {
        record.handlers.remove(name);
    }
    super::event_target::set_attribute_handler(
        scope,
        arguments.this(),
        name.strip_prefix("on").unwrap_or(name),
        present,
    );
}

fn get_onloading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, "onloading")
}
fn set_onloading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onloading")
}
fn get_onloading_done(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, "onloadingdone")
}
fn set_onloading_done(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onloadingdone")
}
fn get_onloading_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, "onloadingerror")
}
fn set_onloading_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onloadingerror")
}
