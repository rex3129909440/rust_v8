use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use v8::{ValueDeserializerHelper, ValueSerializerHelper};

#[derive(Clone)]
pub(crate) struct SerializedMessage {
    bytes: Vec<u8>,
    transferred_buffers: Vec<v8::SharedRef<v8::BackingStore>>,
    shared_buffers: Vec<v8::SharedRef<v8::BackingStore>>,
    pub(crate) ports: Vec<v8::Global<v8::Object>>,
}

struct SerializerDelegate {
    shared_buffers: Rc<RefCell<Vec<v8::SharedRef<v8::BackingStore>>>>,
    transferred_ports: HashMap<i32, u32>,
}

impl v8::ValueSerializerImpl for SerializerDelegate {
    fn throw_data_clone_error<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        message: v8::Local<'s, v8::String>,
    ) {
        throw_data_clone_error(scope, message);
    }

    fn get_shared_array_buffer_id<'s>(
        &self,
        _scope: &mut v8::PinScope<'s, '_>,
        buffer: v8::Local<'s, v8::SharedArrayBuffer>,
    ) -> Option<u32> {
        let mut buffers = self.shared_buffers.borrow_mut();
        let id = u32::try_from(buffers.len()).ok()?;
        buffers.push(buffer.get_backing_store());
        Some(id)
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
                || super::structured_clone::nonserializable_platform_interface(scope, object)
                    .is_some(),
        )
    }

    fn write_host_object<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        object: v8::Local<'s, v8::Object>,
        serializer: &dyn v8::ValueSerializerHelper,
    ) -> Option<bool> {
        if let Some(interface) =
            super::structured_clone::nonserializable_platform_interface(scope, object)
            && !super::message_port::is_port(scope, object)
        {
            throw_data_clone_text(scope, &format!("A {interface} object could not be cloned."));
            return None;
        }
        let Some(transfer_id) = self
            .transferred_ports
            .get(&object.get_identity_hash().get())
            .copied()
        else {
            throw_data_clone_text(
                scope,
                "A MessagePort could not be cloned because it was not transferred",
            );
            return None;
        };
        serializer.write_uint32(transfer_id);
        Some(true)
    }
}

struct DeserializerDelegate {
    shared_buffers: Vec<v8::SharedRef<v8::BackingStore>>,
    transferred_ports: Vec<v8::Global<v8::Object>>,
}

impl v8::ValueDeserializerImpl for DeserializerDelegate {
    fn get_shared_array_buffer_from_id<'s>(
        &self,
        scope: &mut v8::PinScope<'s, '_>,
        transfer_id: u32,
    ) -> Option<v8::Local<'s, v8::SharedArrayBuffer>> {
        let store = self.shared_buffers.get(transfer_id as usize)?;
        Some(v8::SharedArrayBuffer::with_backing_store(scope, store))
    }

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

pub(crate) fn serialize(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    options_or_transfer: v8::Local<'_, v8::Value>,
) -> Result<SerializedMessage, ()> {
    let transfer = transfer_value(scope, options_or_transfer);
    let mut transferred_buffers = Vec::new();
    let mut source_buffers = Vec::new();
    let mut source_ports = Vec::new();
    let mut identities = HashSet::new();
    if let Some(transfer) = transfer {
        for index in 0..transfer.length() {
            let Some(item) = transfer.get_index(scope, index) else {
                continue;
            };
            if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(item) {
                if buffer.was_detached() || !buffer.is_detachable() {
                    throw_data_clone_text(scope, "ArrayBuffer is not transferable");
                    return Err(());
                }
                if !identities.insert(buffer.get_identity_hash().get()) {
                    throw_data_clone_text(scope, "Transfer list contains duplicate objects");
                    return Err(());
                }
                transferred_buffers.push(buffer.get_backing_store());
                source_buffers.push(v8::Global::new(scope, buffer));
                continue;
            }
            if let Ok(port) = v8::Local::<v8::Object>::try_from(item)
                && super::message_port::is_port(scope, port)
            {
                if super::message_port::validate_transfer(scope, port).is_err() {
                    throw_data_clone_text(scope, "MessagePort is not transferable");
                    return Err(());
                }
                if !identities.insert(port.get_identity_hash().get()) {
                    throw_data_clone_text(scope, "Transfer list contains duplicate objects");
                    return Err(());
                }
                source_ports.push(v8::Global::new(scope, port));
                continue;
            }
            throw_data_clone_text(scope, "Value in transfer list is not transferable");
            return Err(());
        }
    }

    let transferred_port_ids = source_ports
        .iter()
        .enumerate()
        .map(|(index, port)| {
            (
                v8::Local::new(scope, port).get_identity_hash().get(),
                index as u32,
            )
        })
        .collect();
    let shared_buffers = Rc::new(RefCell::new(Vec::new()));
    let serializer = v8::ValueSerializer::new(
        scope,
        Box::new(SerializerDelegate {
            shared_buffers: shared_buffers.clone(),
            transferred_ports: transferred_port_ids,
        }),
    );
    serializer.write_header();
    for (index, buffer) in source_buffers.iter().enumerate() {
        serializer.transfer_array_buffer(index as u32, v8::Local::new(scope, buffer));
    }
    if serializer.write_value(scope.get_entered_or_microtask_context(), value) != Some(true) {
        return Err(());
    }
    let bytes = serializer.release();
    drop(serializer);
    for buffer in &source_buffers {
        let _ = v8::Local::new(scope, buffer).detach(None);
    }
    let mut ports = Vec::with_capacity(source_ports.len());
    for source in source_ports {
        let source = v8::Local::new(scope, &source);
        let target = super::message_port::transfer_object(scope, source).map_err(|message| {
            throw_data_clone_text(scope, &message);
        })?;
        ports.push(v8::Global::new(scope, target));
    }
    let shared_buffers = shared_buffers.borrow().clone();
    Ok(SerializedMessage {
        bytes,
        transferred_buffers,
        shared_buffers,
        ports,
    })
}

pub(crate) fn deserialize<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    message: &SerializedMessage,
) -> Option<v8::Local<'s, v8::Value>> {
    for port in &message.ports {
        super::message_port::adopt_transferred_object(scope, v8::Local::new(scope, port));
    }
    let deserializer = v8::ValueDeserializer::new(
        scope,
        Box::new(DeserializerDelegate {
            shared_buffers: message.shared_buffers.clone(),
            transferred_ports: message.ports.clone(),
        }),
        &message.bytes,
    );
    for (index, store) in message.transferred_buffers.iter().enumerate() {
        let buffer = v8::ArrayBuffer::with_backing_store(scope, store);
        deserializer.transfer_array_buffer(index as u32, buffer);
    }
    let context = scope.get_current_context();
    if deserializer.read_header(context) != Some(true) {
        return None;
    }
    deserializer.read_value(context)
}

fn transfer_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    options_or_transfer: v8::Local<'s, v8::Value>,
) -> Option<v8::Local<'s, v8::Array>> {
    if let Ok(array) = v8::Local::<v8::Array>::try_from(options_or_transfer) {
        return Some(array);
    }
    let options = v8::Local::<v8::Object>::try_from(options_or_transfer).ok()?;
    let key = v8::String::new(scope, "transfer")?;
    options
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
}

fn throw_data_clone_text(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    let Some(message) = v8::String::new(scope, message) else {
        return;
    };
    throw_data_clone_error(scope, message);
}

fn throw_data_clone_error(scope: &mut v8::PinScope<'_, '_>, message: v8::Local<'_, v8::String>) {
    let global = scope.get_current_context().global(scope);
    let exception = v8::String::new(scope, "DOMException")
        .and_then(|key| global.get(scope, key.into()))
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .and_then(|constructor| {
            let name = v8::String::new(scope, "DataCloneError")?;
            constructor.new_instance(scope, &[message.into(), name.into()])
        })
        .map(Into::into)
        .unwrap_or_else(|| v8::Exception::error(scope, message));
    scope.throw_exception(exception);
}
