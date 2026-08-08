use super::html_canvas_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "transferControlToOffscreen",
        0,
        transfer_control_to_offscreen,
    )
}

fn transfer_control_to_offscreen(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.transferred || snapshot.context_kind.is_some() {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Cannot transfer control from a canvas that has a context or was already transferred",
        );
        return;
    }
    let offscreen = match super::offscreen_canvas::create(scope, snapshot.width, snapshot.height) {
        Ok(offscreen) => offscreen,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let stored = v8::Global::new(scope, offscreen);
    if let Some(record) = scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.transferred = true;
        record.transferred_canvas = Some(stored);
    }
    result.set(offscreen.into());
}
