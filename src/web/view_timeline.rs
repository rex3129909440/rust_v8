use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ViewTimelineStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ViewTimelineRecord>,
}

#[derive(Clone)]
struct ViewTimelineRecord {
    subject: Option<v8::Global<v8::Object>>,
    start_offset: Option<v8::Global<v8::Value>>,
    end_offset: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ViewTimelineStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ViewTimeline", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ViewTimelineStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ViewTimeline",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "subject", get_subject)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "startOffset", get_start_offset)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "endOffset", get_end_offset)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let scroll_timeline = super::scroll_timeline::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, scroll_timeline)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ViewTimelineStore>()
        .ok_or_else(|| "ViewTimeline state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "ViewTimeline must be constructed with new");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let subject = options.and_then(|options| object_property(scope, options, "subject"));
    let axis = options
        .and_then(|options| string_property(scope, options, "axis"))
        .unwrap_or_else(|| "block".to_owned());
    let object = arguments.this();
    super::scroll_timeline::attach(scope, object, subject, axis);
    let start_offset = options.and_then(|options| value_property(scope, options, "startOffset"));
    let end_offset = options.and_then(|options| value_property(scope, options, "endOffset"));
    let subject = subject.map(|subject| v8::Global::new(scope, subject));
    let start_offset = start_offset.map(|value| v8::Global::new(scope, value));
    let end_offset = end_offset.map(|value| v8::Global::new(scope, value));
    if let Some(store) = scope.get_slot_mut::<ViewTimelineStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            ViewTimelineRecord {
                subject,
                start_offset,
                end_offset,
            },
        );
    }
    result.set(object.into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ViewTimelineRecord> {
    scope
        .get_slot::<ViewTimelineStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Value>>,
    result: &mut v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_subject(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(subject) = record.subject {
        result.set(v8::Local::new(scope, &subject).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_start_offset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, record.start_offset, &mut result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_end_offset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, record.end_offset, &mut result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    value_property(scope, object, name)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
}

fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined() && !value.is_null()).then_some(value)
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    value_property(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}
