use super::html_canvas_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getContext", 1, get_context)
}

fn get_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.transferred {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Cannot get a context after control has been transferred",
        );
        return;
    }
    let requested = crate::webidl::value_to_string(scope, arguments.get(0));
    let kind = match requested.as_str() {
        "2d" => "2d",
        "webgl" | "experimental-webgl" => "webgl",
        "webgl2" => "webgl2",
        "bitmaprenderer" => "bitmaprenderer",
        _ => {
            result.set(v8::null(scope).into());
            return;
        }
    };
    if snapshot
        .context_kind
        .as_ref()
        .is_some_and(|current| current != kind)
    {
        result.set(v8::null(scope).into());
        return;
    }
    if let Some(context) = existing_context(scope, &snapshot, kind) {
        result.set(context.into());
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let created = match kind {
        "2d" => super::canvas_rendering_context_2d::create(scope, arguments.this(), options),
        "webgl" => super::webgl_rendering_context::create(
            scope,
            Some(arguments.this()),
            snapshot.width,
            snapshot.height,
        ),
        "webgl2" => super::webgl2_rendering_context::create(
            scope,
            Some(arguments.this()),
            snapshot.width,
            snapshot.height,
        ),
        "bitmaprenderer" => super::image_bitmap_rendering_context::create(scope, arguments.this()),
        _ => unreachable!(),
    };
    let context = match created {
        Ok(context) => context,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let stored = v8::Global::new(scope, context);
    if let Some(record) = scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.context_kind = Some(kind.to_owned());
        match kind {
            "2d" => record.context_2d = Some(stored),
            "webgl" => record.context_webgl = Some(stored),
            "webgl2" => record.context_webgl2 = Some(stored),
            "bitmaprenderer" => record.context_bitmap = Some(stored),
            _ => {}
        }
    }
    result.set(context.into());
}
