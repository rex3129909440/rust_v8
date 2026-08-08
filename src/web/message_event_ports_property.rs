use super::message_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "ports", get_ports)
}

fn get_ports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let ports = v8::Array::new(scope, record.ports.len() as i32);
    for (index, port) in record.ports.iter().enumerate() {
        let _ = ports.set_index(scope, index as u32, v8::Local::new(scope, port).into());
    }
    result.set(ports.into());
}
