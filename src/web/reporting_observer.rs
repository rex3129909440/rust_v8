use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ReportingObserverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ObserverRecord>,
}

#[derive(Clone)]
struct ObserverRecord {
    callback: v8::Global<v8::Function>,
    observer: v8::Global<v8::Object>,
    active: bool,
    reports: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReportingObserverStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ReportingObserver", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ReportingObserverStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ReportingObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(scope, prototype, "observe", 0, observe)?;
    crate::webidl::define_method(scope, prototype, "takeRecords", 0, take_records)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ReportingObserverStore>()
        .ok_or_else(|| "ReportingObserver state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReportingObserver': use the new operator",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReportingObserver': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let record = ObserverRecord {
        callback: v8::Global::new(scope, callback),
        observer: v8::Global::new(scope, arguments.this()),
        active: false,
        reports: Vec::new(),
    };
    scope
        .get_slot_mut::<ReportingObserverStore>()
        .expect("ReportingObserver state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn disconnect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<ReportingObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.active = false;
        record.reports.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn observe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<ReportingObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.active = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn take_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let reports = {
        let Some(record) = scope
            .get_slot_mut::<ReportingObserverStore>()
            .and_then(|store| {
                store
                    .records
                    .get_mut(&arguments.this().get_identity_hash().get())
            })
        else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        std::mem::take(&mut record.reports)
    };
    let values = v8::Array::new(scope, reports.len() as i32);
    for (index, report) in reports.iter().enumerate() {
        let report = v8::Local::new(scope, report);
        let _ = values.set_index(scope, index as u32, report.into());
    }
    result.set(values.into());
}

pub(crate) fn queue_report(
    scope: &mut v8::PinScope<'_, '_>,
    observer: v8::Local<'_, v8::Object>,
    report: v8::Local<'_, v8::Object>,
) -> bool {
    let identity = observer.get_identity_hash().get();
    let active = scope
        .get_slot::<ReportingObserverStore>()
        .and_then(|store| store.records.get(&identity))
        .is_some_and(|record| record.active);
    if !active {
        return false;
    }
    let report_global = v8::Global::new(scope, report);
    let Some(record) = scope
        .get_slot_mut::<ReportingObserverStore>()
        .and_then(|store| store.records.get_mut(&identity))
    else {
        return false;
    };
    record.reports.push(report_global);
    let callback = record.callback.clone();
    let observer = record.observer.clone();
    let reports = record.reports.clone();
    let values = v8::Array::new(scope, reports.len() as i32);
    for (index, report) in reports.iter().enumerate() {
        let report = v8::Local::new(scope, report);
        let _ = values.set_index(scope, index as u32, report.into());
    }
    let callback = v8::Local::new(scope, &callback);
    let observer = v8::Local::new(scope, &observer);
    let receiver = v8::undefined(scope);
    let _ = callback.call(scope, receiver.into(), &[values.into(), observer.into()]);
    true
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ReportingObserverStore>() {
        store.constructor.remove(realm_id);
    }
}
