#[derive(Default)]
pub(crate) struct GetSelectionStore {
    selection: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GetSelectionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let document = super::document_global::value(scope)
        .ok_or_else(|| "window document is unavailable".to_owned())?;
    let selection = super::selection::for_document(scope, document)?;
    scope
        .get_slot_mut::<GetSelectionStore>()
        .ok_or_else(|| "window selection state was not prepared".to_owned())?
        .selection = Some(v8::Global::new(scope, selection));
    let function = crate::webidl::create_function(
        scope,
        "getSelection",
        0,
        v8::ConstructorBehavior::Throw,
        get_selection,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "getSelection")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.getSelection".to_owned())
    }
}

fn get_selection(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(selection) = scope
        .get_slot::<GetSelectionStore>()
        .and_then(|store| store.selection.as_ref())
        .cloned()
    {
        result.set(v8::Local::new(scope, &selection).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
