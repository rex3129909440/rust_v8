use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextEncoderStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashMap<i32, ()>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextEncoderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextEncoder", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TextEncoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TextEncoder",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "encoding", get_encoding)?;
    crate::webidl::define_method(scope, prototype, "encode", 0, encode)?;
    crate::webidl::define_method(scope, prototype, "encodeInto", 2, encode_into)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TextEncoderStore>()
        .ok_or_else(|| "TextEncoder state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'TextEncoder': use new");
        return;
    }
    let object = arguments.this();
    scope
        .get_slot_mut::<TextEncoderStore>()
        .expect("TextEncoder state")
        .instances
        .insert(object.get_identity_hash().get(), ());
    result.set(object.into());
}

fn valid_receiver(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<TextEncoderStore>().is_some_and(|store| {
        store
            .instances
            .contains_key(&object.get_identity_hash().get())
    })
}

fn get_encoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(scope, "utf-8") {
        result.set(value.into());
    }
}

pub(crate) fn encode_value(value: &str) -> Vec<u8> {
    value.as_bytes().to_vec()
}

pub(crate) fn uint8_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    bytes: Vec<u8>,
) -> Result<v8::Local<'s, v8::Uint8Array>, String> {
    let length = bytes.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    v8::Uint8Array::new(scope, buffer, 0, length)
        .ok_or_else(|| "cannot create Uint8Array".to_owned())
}

fn encode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let text = if arguments.length() == 0 {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    match uint8_array(scope, encode_value(&text)) {
        Ok(array) => result.set(array.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn encode_into(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "encodeInto requires 2 arguments");
        return;
    }
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(destination) = v8::Local::<v8::Uint8Array>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "destination must be a Uint8Array");
        return;
    };
    let capacity = destination.byte_length();
    let mut output = Vec::with_capacity(capacity);
    let mut read = 0_u32;
    for character in text.chars() {
        let mut buffer = [0_u8; 4];
        let encoded = character.encode_utf8(&mut buffer).as_bytes();
        if output.len() + encoded.len() > capacity {
            break;
        }
        output.extend_from_slice(encoded);
        read += character.len_utf16() as u32;
    }
    if !output.is_empty() {
        unsafe {
            std::ptr::copy_nonoverlapping(
                output.as_ptr(),
                destination.data().cast::<u8>(),
                output.len(),
            );
        }
    }
    let answer = v8::Object::new(scope);
    define_data(
        scope,
        answer,
        "read",
        v8::Integer::new_from_unsigned(scope, read).into(),
    );
    define_data(
        scope,
        answer,
        "written",
        v8::Integer::new_from_unsigned(scope, output.len() as u32).into(),
    );
    result.set(answer.into());
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TextEncoderStore>() {
        store.constructor.remove(realm_id);
    }
}
