use super::event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "composedPath", 0, composed_path)
}

fn composed_path(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let path = if record.dispatching {
        let current_index = record.current_target.as_ref().and_then(|current| {
            let current = v8::Local::new(scope, current);
            let current_id = current.get_identity_hash().get();
            record.path.iter().position(|entry| {
                v8::Local::new(scope, entry).get_identity_hash().get() == current_id
            })
        });
        let mut visible_start = 0;
        if let Some(current_index) = current_index {
            for (index, entry) in record.path.iter().enumerate().take(current_index) {
                let entry = v8::Local::new(scope, entry);
                if super::shadow_root::is_closed(scope, entry) {
                    visible_start = index + 1;
                }
            }
        }
        record.path.into_iter().skip(visible_start).collect()
    } else {
        Vec::new()
    };
    let array = v8::Array::new(scope, path.len() as i32);
    for (index, target) in path.iter().enumerate() {
        let target = v8::Local::new(scope, target);
        let _ = array.set_index(scope, index as u32, target.into());
    }
    result.set(array.into());
}
