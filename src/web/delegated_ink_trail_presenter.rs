use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DelegatedInkTrailPresenterStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DelegatedInkTrailPresenterRecord>,
}

#[derive(Clone)]
struct DelegatedInkTrailPresenterRecord {
    presentation_area: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DelegatedInkTrailPresenterStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DelegatedInkTrailPresenter", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<DelegatedInkTrailPresenterStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DelegatedInkTrailPresenter",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "presentationArea",
        get_presentation_area,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "updateInkTrailStartPoint",
        2,
        update_ink_trail_start_point,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DelegatedInkTrailPresenterStore>()
        .ok_or_else(|| "DelegatedInkTrailPresenter state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    presentation_area: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DelegatedInkTrailPresenter".to_owned());
    }
    let presentation_area = presentation_area.map(|area| v8::Global::new(scope, area));
    scope
        .get_slot_mut::<DelegatedInkTrailPresenterStore>()
        .ok_or_else(|| "DelegatedInkTrailPresenter state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            DelegatedInkTrailPresenterRecord { presentation_area },
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'DelegatedInkTrailPresenter': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DelegatedInkTrailPresenterRecord> {
    scope
        .get_slot::<DelegatedInkTrailPresenterStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_presentation_area(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.presentation_area {
            Some(area) => result.set(v8::Local::new(scope, &area).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn update_ink_trail_start_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'updateInkTrailStartPoint': 2 arguments required",
        );
        return;
    }
    let Ok(pointer_event) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'PointerEvent'");
        return;
    };
    if !super::pointer_event::is_pointer_event(scope, pointer_event) {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'PointerEvent'");
        return;
    }
    if !arguments.get(1).is_object() && !arguments.get(1).is_null_or_undefined() {
        crate::webidl::throw_type_error(scope, "The provided value is not of type 'InkTrailStyle'");
        return;
    }
    if let Ok(exception) = super::dom_exception::create(
        scope,
        "Only trusted pointerevents are accepted.".to_owned(),
        "NotAllowedError".to_owned(),
    ) {
        scope.throw_exception(exception.into());
    }
}
