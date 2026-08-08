#[derive(Default)]
pub(crate) struct AlertStore {
    last_message: Option<String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AlertStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "alert", 0, v8::ConstructorBehavior::Throw, alert)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "alert")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.alert".to_owned())
    }
}
fn alert(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let message = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<AlertStore>() {
        store.last_message = Some(message);
    }
}
