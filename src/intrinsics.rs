#[allow(dead_code)]
pub(crate) struct LateIntrinsics {
    pub temporal: v8::Global<v8::Value>,
    pub suppressed_error: v8::Global<v8::Value>,
    pub disposable_stack: v8::Global<v8::Value>,
    pub async_disposable_stack: v8::Global<v8::Value>,
    pub float16_array: v8::Global<v8::Value>,
    pub web_assembly: v8::Global<v8::Value>,
}

impl LateIntrinsics {
    pub(crate) fn detach(
        scope: &v8::PinScope<'_, '_>,
        context: v8::Local<'_, v8::Context>,
    ) -> Result<Self, String> {
        let global = context.global(scope);

        let temporal = take(scope, global, "Temporal")?;
        let suppressed_error = take(scope, global, "SuppressedError")?;
        let disposable_stack = take(scope, global, "DisposableStack")?;
        let async_disposable_stack = take(scope, global, "AsyncDisposableStack")?;
        let float16_array = take(scope, global, "Float16Array")?;
        remove(scope, global, "SharedArrayBuffer")?;
        let web_assembly = take(scope, global, "WebAssembly")?;

        Ok(Self {
            temporal,
            suppressed_error,
            disposable_stack,
            async_disposable_stack,
            float16_array,
            web_assembly,
        })
    }
}

fn take(
    scope: &v8::PinScope<'_, '_>,
    global: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<v8::Global<v8::Value>, String> {
    let key =
        v8::String::new(scope, name).ok_or_else(|| format!("invalid intrinsic name {name}"))?;
    let value = global
        .get(scope, key.into())
        .ok_or_else(|| format!("missing V8 intrinsic {name}"))?;
    if global.delete(scope, key.into()) != Some(true) {
        return Err(format!("cannot detach V8 intrinsic {name}"));
    }
    Ok(v8::Global::new(scope, value))
}

fn remove(
    scope: &v8::PinScope<'_, '_>,
    global: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let key =
        v8::String::new(scope, name).ok_or_else(|| format!("invalid intrinsic name {name}"))?;
    if global.delete(scope, key.into()) != Some(true) {
        return Err(format!("cannot remove V8 intrinsic {name}"));
    }
    Ok(())
}
