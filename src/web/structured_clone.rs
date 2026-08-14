use std::collections::{HashMap, HashSet};

use v8::{ValueDeserializerHelper, ValueSerializerHelper};

struct PlatformObjectRecord {
    value: v8::Global<v8::Object>,
    interface: String,
    serializable: bool,
}

#[derive(Default)]
struct PlatformCloneRegistry {
    prototypes: HashMap<i32, Vec<PlatformObjectRecord>>,
    objects: HashMap<i32, Vec<PlatformObjectRecord>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PlatformCloneRegistry::default());
}

pub(crate) fn register_platform_prototype(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    interface: &str,
) {
    let record = PlatformObjectRecord {
        value: v8::Global::new(scope, prototype),
        interface: interface.to_owned(),
        serializable: serializable_platform_interface(interface),
    };
    insert_platform_record(scope, prototype, record, true);
}

pub(crate) fn register_platform_object_from_prototype(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    prototype: v8::Local<'_, v8::Value>,
) {
    let Ok(prototype) = v8::Local::<v8::Object>::try_from(prototype) else {
        return;
    };
    let Some((interface, serializable)) = platform_record(scope, prototype, true) else {
        return;
    };
    let record = PlatformObjectRecord {
        value: v8::Global::new(scope, object),
        interface,
        serializable,
    };
    insert_platform_record(scope, object, record, false);
}

pub(crate) fn register_constructed_platform_object(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) {
    if let Some(prototype) = object.get_prototype(scope) {
        register_platform_object_from_prototype(scope, object, prototype);
    }
}

pub(crate) fn nonserializable_platform_interface(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    platform_record(scope, object, false)
        .and_then(|(interface, serializable)| (!serializable).then_some(interface))
}

pub(crate) fn inherits_platform_interface(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    interface: &str,
) -> bool {
    if platform_record(scope, object, false).is_some_and(|(current, _)| current == interface) {
        return true;
    }
    let mut current = object.get_prototype(scope);
    for _ in 0..32 {
        let Some(value) = current else {
            break;
        };
        let Ok(prototype) = v8::Local::<v8::Object>::try_from(value) else {
            break;
        };
        if platform_record(scope, prototype, true).is_some_and(|(current, _)| current == interface)
        {
            return true;
        }
        current = prototype.get_prototype(scope);
    }
    false
}

fn insert_platform_record(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Object>,
    record: PlatformObjectRecord,
    prototype: bool,
) {
    if platform_record(scope, value, prototype).is_some() {
        return;
    }
    let hash = value.get_identity_hash().get();
    let Some(registry) = scope.get_slot_mut::<PlatformCloneRegistry>() else {
        return;
    };
    let records = if prototype {
        &mut registry.prototypes
    } else {
        &mut registry.objects
    };
    records.entry(hash).or_default().push(record);
}

fn platform_record(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Object>,
    prototype: bool,
) -> Option<(String, bool)> {
    let hash = value.get_identity_hash().get();
    let registry = scope.get_slot::<PlatformCloneRegistry>()?;
    let records = if prototype {
        &registry.prototypes
    } else {
        &registry.objects
    };
    records.get(&hash)?.iter().find_map(|record| {
        v8::Local::new(scope, &record.value)
            .strict_equals(value.into())
            .then(|| (record.interface.clone(), record.serializable))
    })
}

fn serializable_platform_interface(interface: &str) -> bool {
    matches!(
        interface,
        "AudioData"
            | "Blob"
            | "CropTarget"
            | "CryptoKey"
            | "DOMException"
            | "DOMMatrix"
            | "DOMMatrixReadOnly"
            | "DOMPoint"
            | "DOMPointReadOnly"
            | "DOMQuad"
            | "DOMRect"
            | "DOMRectReadOnly"
            | "EncodedAudioChunk"
            | "EncodedVideoChunk"
            | "File"
            | "FileList"
            | "FileSystemDirectoryHandle"
            | "FileSystemFileHandle"
            | "ImageBitmap"
            | "ImageData"
            | "RTCCertificate"
            | "VideoFrame"
    )
}

struct CloneSerializer {
    transferred_ports: HashMap<i32, u32>,
}

impl v8::ValueSerializerImpl for CloneSerializer {
    fn throw_data_clone_error<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        message: v8::Local<'s, v8::String>,
    ) {
        throw_data_clone_error(scope, &message.to_rust_string_lossy(scope));
    }

    fn has_custom_host_object(&self, _isolate: &v8::Isolate) -> bool {
        true
    }

    fn is_host_object<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        object: v8::Local<'s, v8::Object>,
    ) -> Option<bool> {
        Some(
            super::message_port::is_port(scope, object)
                || nonserializable_platform_interface(scope, object).is_some(),
        )
    }

    fn write_host_object<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        object: v8::Local<'s, v8::Object>,
        serializer: &dyn v8::ValueSerializerHelper,
    ) -> Option<bool> {
        if let Some(interface) = nonserializable_platform_interface(scope, object)
            && !super::message_port::is_port(scope, object)
        {
            throw_data_clone_error(scope, &format!("A {interface} object could not be cloned."));
            return None;
        }
        let Some(transfer_id) = self
            .transferred_ports
            .get(&object.get_identity_hash().get())
            .copied()
        else {
            throw_data_clone_error(
                scope,
                "A MessagePort could not be cloned because it was not transferred.",
            );
            return None;
        };
        serializer.write_uint32(transfer_id);
        Some(true)
    }
}

struct CloneDeserializer {
    transferred_ports: Vec<v8::Global<v8::Object>>,
}

impl v8::ValueDeserializerImpl for CloneDeserializer {
    fn read_host_object<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        deserializer: &dyn v8::ValueDeserializerHelper,
    ) -> Option<v8::Local<'s, v8::Object>> {
        let mut transfer_id = 0;
        if !deserializer.read_uint32(&mut transfer_id) {
            return None;
        }
        self.transferred_ports
            .get(transfer_id as usize)
            .map(|port| v8::Local::new(scope, port))
    }
}

#[derive(Default)]
pub(crate) struct TransferList {
    array_buffers: Vec<v8::Global<v8::ArrayBuffer>>,
    message_ports: Vec<v8::Global<v8::Object>>,
    message_port_identities: HashSet<i32>,
}

impl TransferList {
    pub(crate) fn contains_message_port(&self, identity: i32) -> bool {
        self.message_port_identities.contains(&identity)
    }
}

pub(crate) struct CloneOutput {
    pub(crate) value: v8::Global<v8::Value>,
    pub(crate) ports: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn transfer_from_options(
    scope: &mut v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Value>,
) -> Result<TransferList, String> {
    let options = crate::trace::unwrap_traced_value(scope, options);
    if options.is_undefined() || options.is_null() {
        return Ok(TransferList::default());
    }
    let options = v8::Local::<v8::Object>::try_from(options)
        .map_err(|_| "The options argument is not an object.".to_owned())?;
    let key = crate::webidl::string(scope, "transfer")?;
    let transfer = options
        .get(scope, key.into())
        .unwrap_or_else(|| v8::undefined(scope).into());
    transfer_from_sequence(scope, transfer)
}

pub(crate) fn transfer_from_sequence(
    scope: &mut v8::PinScope<'_, '_>,
    transfer: v8::Local<'_, v8::Value>,
) -> Result<TransferList, String> {
    let transfer = crate::trace::unwrap_traced_value(scope, transfer);
    if transfer.is_undefined() || transfer.is_null() {
        return Ok(TransferList::default());
    }
    let array = v8::Local::<v8::Array>::try_from(transfer)
        .map_err(|_| "The transfer list must be an Array.".to_owned())?;
    let mut output = TransferList::default();
    let mut identities = HashSet::new();
    for index in 0..array.length() {
        let value = array
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let value = crate::trace::unwrap_traced_value(scope, value);
        if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
            let identity = buffer.get_identity_hash().get();
            if !identities.insert(identity) {
                return Err("Transfer list contains duplicate transferable objects.".to_owned());
            }
            if buffer.was_detached() || !buffer.is_detachable() {
                return Err("An ArrayBuffer in the transfer list is not transferable.".to_owned());
            }
            output.array_buffers.push(v8::Global::new(scope, buffer));
            continue;
        }
        if let Ok(port) = v8::Local::<v8::Object>::try_from(value)
            && super::message_port::is_port(scope, port)
        {
            let identity = port.get_identity_hash().get();
            if !identities.insert(identity) {
                return Err("Transfer list contains duplicate transferable objects.".to_owned());
            }
            super::message_port::validate_transfer(scope, port)?;
            output.message_ports.push(v8::Global::new(scope, port));
            output.message_port_identities.insert(identity);
            continue;
        }
        return Err("An object in the transfer list is not transferable.".to_owned());
    }
    Ok(output)
}

pub(crate) fn clone_into(
    scope: &mut v8::PinScope<'_, '_>,
    target_context: v8::Local<'_, v8::Context>,
    value: v8::Local<'_, v8::Value>,
    transfer: TransferList,
) -> Result<CloneOutput, String> {
    let value = v8::Global::new(scope, value);
    let value = v8::Local::new(scope, &value);
    let value = materialize_untraced_graph(scope, value)?;
    let source_context = scope.get_entered_or_microtask_context();
    let transferred_ports = transfer
        .message_ports
        .iter()
        .enumerate()
        .map(|(index, port)| {
            (
                v8::Local::new(scope, port).get_identity_hash().get(),
                index as u32,
            )
        })
        .collect::<HashMap<_, _>>();
    let serializer =
        v8::ValueSerializer::new(scope, Box::new(CloneSerializer { transferred_ports }));
    for (index, buffer) in transfer.array_buffers.iter().enumerate() {
        serializer.transfer_array_buffer(index as u32, v8::Local::new(scope, buffer));
    }
    serializer.write_header();
    if serializer.write_value(source_context, value) != Some(true) {
        return Err("The object could not be cloned.".to_owned());
    }
    let serialized = serializer.release();
    drop(serializer);

    let source_buffers = transfer.array_buffers;
    let source_ports = transfer.message_ports;
    let cloned = {
        let target_scope = &mut v8::ContextScope::new(scope, target_context);
        let mut target_buffers = Vec::with_capacity(source_buffers.len());
        for source in &source_buffers {
            let source = v8::Local::new(target_scope, source);
            let backing_store = source.get_backing_store();
            target_buffers.push(v8::ArrayBuffer::with_backing_store(
                target_scope,
                &backing_store,
            ));
        }
        let mut target_ports = Vec::with_capacity(source_ports.len());
        for source in &source_ports {
            let source = v8::Local::new(target_scope, source);
            target_ports.push(super::message_port::transfer_object(target_scope, source)?);
        }
        let target_port_globals = target_ports
            .iter()
            .map(|port| v8::Global::new(target_scope, *port))
            .collect::<Vec<_>>();
        let deserializer = v8::ValueDeserializer::new(
            target_scope,
            Box::new(CloneDeserializer {
                transferred_ports: target_port_globals.clone(),
            }),
            &serialized,
        );
        for (index, buffer) in target_buffers.iter().enumerate() {
            deserializer.transfer_array_buffer(index as u32, *buffer);
        }
        if deserializer.read_header(target_context) != Some(true) {
            return Err("Cannot read structured clone payload.".to_owned());
        }
        let value = deserializer
            .read_value(target_context)
            .ok_or_else(|| "Cannot deserialize structured clone payload.".to_owned())?;
        Ok::<_, String>((v8::Global::new(target_scope, value), target_port_globals))
    }?;

    for buffer in source_buffers {
        let buffer = v8::Local::new(scope, &buffer);
        if buffer.detach(None) != Some(true) {
            return Err("An ArrayBuffer could not be detached.".to_owned());
        }
    }
    Ok(CloneOutput {
        value: cloned.0,
        ports: cloned.1,
    })
}

#[derive(Default)]
struct MaterializedGraph {
    values: Vec<(v8::Global<v8::Value>, v8::Global<v8::Value>)>,
}

impl MaterializedGraph {
    fn existing<'s>(
        &self,
        scope: &v8::PinScope<'s, '_>,
        source: v8::Local<'s, v8::Value>,
    ) -> Option<v8::Local<'s, v8::Value>> {
        self.values.iter().find_map(|(candidate, materialized)| {
            v8::Local::new(scope, candidate)
                .strict_equals(source)
                .then(|| v8::Local::new(scope, materialized))
        })
    }

    fn insert(
        &mut self,
        scope: &v8::PinScope<'_, '_>,
        source: v8::Local<'_, v8::Value>,
        materialized: v8::Local<'_, v8::Value>,
    ) {
        self.values.push((
            v8::Global::new(scope, source),
            v8::Global::new(scope, materialized),
        ));
    }
}

fn materialize_untraced_graph<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Result<v8::Local<'s, v8::Value>, String> {
    if !crate::trace::is_enabled(scope) {
        return Ok(value);
    }
    materialize_value(scope, value, &mut MaterializedGraph::default())
}

fn materialize_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    graph: &mut MaterializedGraph,
) -> Result<v8::Local<'s, v8::Value>, String> {
    let value = crate::trace::unwrap_traced_value(scope, value);
    if !value.is_object() {
        return Ok(value);
    }
    if let Some(existing) = graph.existing(scope, value) {
        return Ok(existing);
    }

    if let Ok(source) = v8::Local::<v8::Map>::try_from(value) {
        let destination = v8::Map::new(scope);
        graph.insert(scope, value, destination.into());
        let entries = source.as_array(scope);
        for index in (0..entries.length()).step_by(2) {
            let key = entries
                .get_index(scope, index)
                .ok_or_else(|| "Cannot read a Map key while cloning.".to_owned())?;
            let entry = entries
                .get_index(scope, index + 1)
                .ok_or_else(|| "Cannot read a Map value while cloning.".to_owned())?;
            let key = materialize_value(scope, key, graph)?;
            let entry = materialize_value(scope, entry, graph)?;
            destination
                .set(scope, key, entry)
                .ok_or_else(|| "Cannot materialize a Map while cloning.".to_owned())?;
        }
        return Ok(destination.into());
    }

    if let Ok(source) = v8::Local::<v8::Set>::try_from(value) {
        let destination = v8::Set::new(scope);
        graph.insert(scope, value, destination.into());
        let entries = source.as_array(scope);
        for index in 0..entries.length() {
            let entry = entries
                .get_index(scope, index)
                .ok_or_else(|| "Cannot read a Set value while cloning.".to_owned())?;
            let entry = materialize_value(scope, entry, graph)?;
            destination
                .add(scope, entry)
                .ok_or_else(|| "Cannot materialize a Set while cloning.".to_owned())?;
        }
        return Ok(destination.into());
    }

    let source = v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "Cannot inspect an object while cloning.".to_owned())?;
    if !is_plain_object(scope, source) && !value.is_array() {
        return Ok(value);
    }
    let destination = if let Ok(array) = v8::Local::<v8::Array>::try_from(value) {
        v8::Local::<v8::Object>::from(v8::Array::new(scope, array.length() as i32))
    } else {
        v8::Object::new(scope)
    };
    graph.insert(scope, value, destination.into());
    let names = source
        .get_own_property_names(
            scope,
            v8::GetPropertyNamesArgs {
                property_filter: v8::PropertyFilter::ONLY_ENUMERABLE
                    | v8::PropertyFilter::SKIP_SYMBOLS,
                key_conversion: v8::KeyConversionMode::ConvertToString,
                ..Default::default()
            },
        )
        .ok_or_else(|| "Cannot enumerate an object while cloning.".to_owned())?;
    for index in 0..names.length() {
        let key = names
            .get_index(scope, index)
            .ok_or_else(|| "Cannot read an object key while cloning.".to_owned())?;
        let key = v8::Local::<v8::Name>::try_from(key)
            .map_err(|_| "Cannot convert an object key while cloning.".to_owned())?;
        let entry = source
            .get(scope, key.into())
            .ok_or_else(|| "Cannot read an object value while cloning.".to_owned())?;
        let entry = materialize_value(scope, entry, graph)?;
        destination
            .create_data_property(scope, key, entry)
            .filter(|created| *created)
            .ok_or_else(|| "Cannot materialize an object while cloning.".to_owned())?;
    }
    Ok(destination.into())
}

fn is_plain_object(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    let Some(prototype) = object.get_prototype(scope) else {
        return false;
    };
    if prototype.is_null() {
        return true;
    }
    let global = scope.get_current_context().global(scope);
    let Some(object_key) = v8::String::new(scope, "Object") else {
        return false;
    };
    let Some(constructor) = global.get(scope, object_key.into()) else {
        return false;
    };
    let constructor = crate::trace::unwrap_traced_value(scope, constructor);
    let Ok(constructor) = v8::Local::<v8::Object>::try_from(constructor) else {
        return false;
    };
    let Some(prototype_key) = v8::String::new(scope, "prototype") else {
        return false;
    };
    let Some(expected) = constructor.get(scope, prototype_key.into()) else {
        return false;
    };
    let expected = crate::trace::unwrap_traced_value(scope, expected);
    let prototype = crate::trace::unwrap_traced_value(scope, prototype);
    prototype.strict_equals(expected)
}

pub(crate) fn throw_data_clone_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "DataCloneError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}
