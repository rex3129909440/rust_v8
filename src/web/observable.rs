use std::collections::HashMap;

#[derive(Clone)]
enum ObservableSource {
    Callback(v8::Global<v8::Function>),
    Values(Vec<v8::Global<v8::Value>>),
    Promise(v8::Global<v8::Promise>),
    Operation {
        upstream: v8::Global<v8::Object>,
        operation: ObservableOperation,
    },
}

#[derive(Clone)]
enum ObservableOperation {
    Catch(v8::Global<v8::Function>),
    Drop(u64),
    Filter(v8::Global<v8::Function>),
    Finally(v8::Global<v8::Function>),
    FlatMap(v8::Global<v8::Function>),
    Inspect(Option<v8::Global<v8::Object>>),
    Map(v8::Global<v8::Function>),
    SwitchMap(v8::Global<v8::Function>),
    Take(u64),
    TakeUntil(v8::Global<v8::Object>),
}

#[derive(Default)]
struct Collector {
    values: Vec<v8::Global<v8::Value>>,
    error: Option<v8::Global<v8::Value>>,
    completed: bool,
}

#[derive(Default)]
pub(crate) struct ObservableStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ObservableSource>,
    next_collector: u32,
    collectors: HashMap<u32, Collector>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ObservableStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Observable", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ObservableStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Observable",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "catch", 1, catch_op)?;
    crate::webidl::define_method(scope, prototype, "drop", 1, drop_op)?;
    crate::webidl::define_method(scope, prototype, "every", 1, every)?;
    crate::webidl::define_method(scope, prototype, "filter", 1, filter_op)?;
    crate::webidl::define_method(scope, prototype, "finally", 1, finally_op)?;
    crate::webidl::define_method(scope, prototype, "find", 1, find)?;
    crate::webidl::define_method(scope, prototype, "first", 0, first)?;
    crate::webidl::define_method(scope, prototype, "flatMap", 1, flat_map)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "inspect", 0, inspect)?;
    crate::webidl::define_method(scope, prototype, "last", 0, last)?;
    crate::webidl::define_method(scope, prototype, "map", 1, map_op)?;
    crate::webidl::define_method(scope, prototype, "reduce", 1, reduce)?;
    crate::webidl::define_method(scope, prototype, "some", 1, some)?;
    crate::webidl::define_method(scope, prototype, "subscribe", 0, subscribe)?;
    crate::webidl::define_method(scope, prototype, "switchMap", 1, switch_map)?;
    crate::webidl::define_method(scope, prototype, "take", 1, take_op)?;
    crate::webidl::define_method(scope, prototype, "takeUntil", 1, take_until)?;
    crate::webidl::define_method(scope, prototype, "toArray", 0, to_array)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "from", 1, from)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ObservableStore>()
        .ok_or_else(|| "Observable state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'Observable': 1 argument required",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'Observable': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let callback = v8::Global::new(scope, callback);
    scope
        .get_slot_mut::<ObservableStore>()
        .expect("Observable state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ObservableSource::Callback(callback),
        );
    result.set(arguments.this().into());
}

fn source(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ObservableSource> {
    scope
        .get_slot::<ObservableStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn create_with_source<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: ObservableSource,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let observable = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, observable, prototype.into()) != Some(true) {
        return Err("cannot create Observable".to_owned());
    }
    scope
        .get_slot_mut::<ObservableStore>()
        .ok_or_else(|| "Observable state was not prepared".to_owned())?
        .records
        .insert(observable.get_identity_hash().get(), source);
    Ok(observable)
}

fn from(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "Cannot convert value to an Observable");
        return;
    };
    if source(scope, object).is_some() {
        result.set(object.into());
        return;
    }
    if let Ok(promise) = v8::Local::<v8::Promise>::try_from(value) {
        match create_with_source(
            scope,
            ObservableSource::Promise(v8::Global::new(scope, promise)),
        ) {
            Ok(observable) => result.set(observable.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    if value.is_string() || value.is_string_object() {
        crate::webidl::throw_type_error(scope, "Cannot convert value to an Observable");
        return;
    }
    let iterator_symbol = v8::Symbol::get_iterator(scope);
    let iterator_method = object
        .get(scope, iterator_symbol.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok());
    let Some(iterator_method) = iterator_method else {
        crate::webidl::throw_type_error(scope, "Cannot convert value to an Observable");
        return;
    };
    let Some(iterator_value) = iterator_method.call(scope, object.into(), &[]) else {
        return;
    };
    let Ok(iterator) = v8::Local::<v8::Object>::try_from(iterator_value) else {
        crate::webidl::throw_type_error(scope, "The iterator is invalid");
        return;
    };
    let Some(next_key) = v8::String::new(scope, "next") else {
        return;
    };
    let Some(next) = iterator
        .get(scope, next_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        crate::webidl::throw_type_error(scope, "The iterator has no next method");
        return;
    };
    let mut values = Vec::new();
    loop {
        let Some(step_value) = next.call(scope, iterator.into(), &[]) else {
            return;
        };
        let Ok(step) = v8::Local::<v8::Object>::try_from(step_value) else {
            break;
        };
        let Some(done_key) = v8::String::new(scope, "done") else {
            break;
        };
        if step
            .get(scope, done_key.into())
            .is_some_and(|value| value.boolean_value(scope))
        {
            break;
        }
        let Some(value_key) = v8::String::new(scope, "value") else {
            break;
        };
        if let Some(item) = step.get(scope, value_key.into()) {
            values.push(v8::Global::new(scope, item));
        }
    }
    match create_with_source(scope, ObservableSource::Values(values)) {
        Ok(observable) => result.set(observable.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn observer_from_argument(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> v8::Global<v8::Object> {
    if let Ok(observer) = v8::Local::<v8::Object>::try_from(value) {
        v8::Global::new(scope, observer)
    } else {
        let observer = v8::Object::new(scope);
        v8::Global::new(scope, observer)
    }
}

fn call_subscriber(
    scope: &mut v8::PinScope<'_, '_>,
    subscriber: v8::Local<'_, v8::Object>,
    method: &str,
    arguments: &[v8::Local<v8::Value>],
) {
    let Some(key) = v8::String::new(scope, method) else {
        return;
    };
    if let Some(function) = subscriber
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    {
        let _ = function.call(scope, subscriber.into(), arguments);
    }
}

fn subscribe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(source) = source(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let observer = observer_from_argument(scope, arguments.get(0));
    let observer = v8::Local::new(scope, &observer);
    let subscriber = match super::subscriber::create(scope, observer) {
        Ok(subscriber) => subscriber,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    emit_source(scope, source, subscriber);
}

fn emit_source(
    scope: &mut v8::PinScope<'_, '_>,
    source: ObservableSource,
    subscriber: v8::Local<'_, v8::Object>,
) {
    match source {
        ObservableSource::Callback(callback) => {
            let callback = v8::Local::new(scope, &callback);
            if let Some(teardown) =
                callback.call(scope, v8::undefined(scope).into(), &[subscriber.into()])
                && let Ok(teardown) = v8::Local::<v8::Function>::try_from(teardown)
            {
                call_subscriber(scope, subscriber, "addTeardown", &[teardown.into()]);
            }
        }
        ObservableSource::Values(values) => {
            for value in values {
                let value = v8::Local::new(scope, &value);
                call_subscriber(scope, subscriber, "next", &[value]);
            }
            call_subscriber(scope, subscriber, "complete", &[]);
        }
        ObservableSource::Promise(promise) => {
            let on_fulfilled = v8::Function::builder(promise_fulfilled)
                .data(subscriber.into())
                .constructor_behavior(v8::ConstructorBehavior::Throw)
                .build(scope);
            let on_rejected = v8::Function::builder(promise_rejected)
                .data(subscriber.into())
                .constructor_behavior(v8::ConstructorBehavior::Throw)
                .build(scope);
            if let (Some(on_fulfilled), Some(on_rejected)) = (on_fulfilled, on_rejected) {
                let promise = v8::Local::new(scope, &promise);
                let _ = promise.then2(scope, on_fulfilled, on_rejected);
            }
        }
        ObservableSource::Operation {
            upstream,
            operation,
        } => {
            let upstream = v8::Local::new(scope, &upstream);
            let mut collected = collect(scope, upstream);
            apply_operation(scope, &mut collected, operation);
            for value in collected.values {
                let value = v8::Local::new(scope, &value);
                call_subscriber(scope, subscriber, "next", &[value]);
            }
            if let Some(error) = collected.error {
                let error = v8::Local::new(scope, &error);
                call_subscriber(scope, subscriber, "error", &[error]);
            } else {
                call_subscriber(scope, subscriber, "complete", &[]);
            }
        }
    }
}

fn promise_fulfilled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Ok(subscriber) = v8::Local::<v8::Object>::try_from(arguments.data()) {
        call_subscriber(scope, subscriber, "next", &[arguments.get(0)]);
        call_subscriber(scope, subscriber, "complete", &[]);
    }
}
fn promise_rejected(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Ok(subscriber) = v8::Local::<v8::Object>::try_from(arguments.data()) {
        call_subscriber(scope, subscriber, "error", &[arguments.get(0)]);
    }
}

fn collect(scope: &mut v8::PinScope<'_, '_>, observable: v8::Local<'_, v8::Object>) -> Collector {
    let collector_id = {
        let store = scope
            .get_slot_mut::<ObservableStore>()
            .expect("Observable state");
        store.next_collector = store.next_collector.wrapping_add(1).max(1);
        let id = store.next_collector;
        store.collectors.insert(id, Collector::default());
        id
    };
    let observer = v8::Object::new(scope);
    let data = v8::Integer::new_from_unsigned(scope, collector_id);
    let next = v8::Function::builder(collector_next)
        .data(data.into())
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    let error = v8::Function::builder(collector_error)
        .data(data.into())
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    let complete = v8::Function::builder(collector_complete)
        .data(data.into())
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    if let Some(next) = next {
        define(scope, observer, "next", next.into());
    }
    if let Some(error) = error {
        define(scope, observer, "error", error.into());
    }
    if let Some(complete) = complete {
        define(scope, observer, "complete", complete.into());
    }
    if let Some(source) = source(scope, observable)
        && let Ok(subscriber) = super::subscriber::create(scope, observer)
    {
        emit_source(scope, source, subscriber);
    }
    scope
        .get_slot_mut::<ObservableStore>()
        .and_then(|store| store.collectors.remove(&collector_id))
        .unwrap_or_default()
}

fn collector_id(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> u32 {
    arguments.data().uint32_value(scope).unwrap_or(0)
}
fn collector_next(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = collector_id(scope, &arguments);
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(collector) = scope
        .get_slot_mut::<ObservableStore>()
        .and_then(|store| store.collectors.get_mut(&id))
    {
        collector.values.push(value)
    }
}
fn collector_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = collector_id(scope, &arguments);
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(collector) = scope
        .get_slot_mut::<ObservableStore>()
        .and_then(|store| store.collectors.get_mut(&id))
    {
        collector.error = Some(value)
    }
}
fn collector_complete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = collector_id(scope, &arguments);
    if let Some(collector) = scope
        .get_slot_mut::<ObservableStore>()
        .and_then(|store| store.collectors.get_mut(&id))
    {
        collector.completed = true
    }
}

fn callback_result<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    callback: &v8::Global<v8::Function>,
    arguments: &[v8::Local<'s, v8::Value>],
) -> Option<v8::Local<'s, v8::Value>> {
    v8::Local::new(scope, callback).call(scope, v8::undefined(scope).into(), arguments)
}

fn apply_operation(
    scope: &mut v8::PinScope<'_, '_>,
    collected: &mut Collector,
    operation: ObservableOperation,
) {
    match operation {
        ObservableOperation::Map(callback) => {
            let input = std::mem::take(&mut collected.values);
            collected.values = input
                .into_iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    let value = v8::Local::new(scope, &value);
                    let index = v8::Integer::new_from_unsigned(scope, index as u32);
                    callback_result(scope, &callback, &[value, index.into()])
                        .map(|mapped| v8::Global::new(scope, mapped))
                })
                .collect();
        }
        ObservableOperation::Filter(callback) => {
            let input = std::mem::take(&mut collected.values);
            collected.values = input
                .into_iter()
                .enumerate()
                .filter(|(index, value)| {
                    let value = v8::Local::new(scope, value);
                    let index = v8::Integer::new_from_unsigned(scope, *index as u32);
                    callback_result(scope, &callback, &[value, index.into()])
                        .is_some_and(|keep| keep.boolean_value(scope))
                })
                .map(|(_, value)| value)
                .collect();
        }
        ObservableOperation::Drop(count) => {
            collected.values = std::mem::take(&mut collected.values)
                .into_iter()
                .skip(count as usize)
                .collect();
        }
        ObservableOperation::Take(count) => {
            collected.values.truncate(count as usize);
        }
        ObservableOperation::Finally(callback) => {
            let _ = callback_result(scope, &callback, &[]);
        }
        ObservableOperation::Inspect(observer) => {
            if let Some(observer) = observer {
                let observer = v8::Local::new(scope, &observer);
                for value in &collected.values {
                    call_observer_method(scope, observer, "next", &[v8::Local::new(scope, value)]);
                }
                if let Some(error) = &collected.error {
                    call_observer_method(scope, observer, "error", &[v8::Local::new(scope, error)]);
                } else {
                    call_observer_method(scope, observer, "complete", &[]);
                }
            }
        }
        ObservableOperation::Catch(callback) => {
            if let Some(error) = collected.error.take() {
                let error = v8::Local::new(scope, &error);
                if let Some(replacement) = callback_result(scope, &callback, &[error])
                    && let Ok(replacement) = v8::Local::<v8::Object>::try_from(replacement)
                    && source(scope, replacement).is_some()
                {
                    *collected = collect(scope, replacement);
                }
            }
        }
        ObservableOperation::FlatMap(callback) => {
            let input = std::mem::take(&mut collected.values);
            let mut output = Vec::new();
            for (index, value) in input.into_iter().enumerate() {
                let value = v8::Local::new(scope, &value);
                let index = v8::Integer::new_from_unsigned(scope, index as u32);
                if let Some(inner) = callback_result(scope, &callback, &[value, index.into()])
                    && let Ok(inner) = v8::Local::<v8::Object>::try_from(inner)
                    && source(scope, inner).is_some()
                {
                    let inner = collect(scope, inner);
                    output.extend(inner.values);
                    if inner.error.is_some() {
                        collected.error = inner.error;
                        break;
                    }
                }
            }
            collected.values = output;
        }
        ObservableOperation::SwitchMap(callback) => {
            let last = collected.values.last().cloned();
            collected.values.clear();
            if let Some(value) = last {
                let value = v8::Local::new(scope, &value);
                if let Some(inner) = callback_result(scope, &callback, &[value])
                    && let Ok(inner) = v8::Local::<v8::Object>::try_from(inner)
                    && source(scope, inner).is_some()
                {
                    *collected = collect(scope, inner);
                }
            }
        }
        ObservableOperation::TakeUntil(notifier) => {
            let notifier = v8::Local::new(scope, &notifier);
            if !collect(scope, notifier).values.is_empty() {
                collected.values.clear();
            }
        }
    }
}

fn call_observer_method(
    scope: &mut v8::PinScope<'_, '_>,
    observer: v8::Local<'_, v8::Object>,
    name: &str,
    arguments: &[v8::Local<v8::Value>],
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    if let Some(function) = observer
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    {
        let _ = function.call(scope, observer.into(), arguments);
    }
}

fn operation(
    scope: &mut v8::PinScope<'_, '_>,
    this: v8::Local<'_, v8::Object>,
    operation: ObservableOperation,
    mut result: v8::ReturnValue<'_>,
) {
    if source(scope, this).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match create_with_source(
        scope,
        ObservableSource::Operation {
            upstream: v8::Global::new(scope, this),
            operation,
        },
    ) {
        Ok(observable) => result.set(observable.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn required_callback(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Global<v8::Function>> {
    match v8::Local::<v8::Function>::try_from(value) {
        Ok(function) => Some(v8::Global::new(scope, function)),
        Err(_) => {
            crate::webidl::throw_type_error(scope, &format!("{name} requires a callback"));
            None
        }
    }
}
fn catch_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(f) = required_callback(s, a.get(0), "catch") {
        operation(s, a.this(), ObservableOperation::Catch(f), r)
    }
}
fn filter_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(f) = required_callback(s, a.get(0), "filter") {
        operation(s, a.this(), ObservableOperation::Filter(f), r)
    }
}
fn finally_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(f) = required_callback(s, a.get(0), "finally") {
        operation(s, a.this(), ObservableOperation::Finally(f), r)
    }
}
fn flat_map(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(f) = required_callback(s, a.get(0), "flatMap") {
        operation(s, a.this(), ObservableOperation::FlatMap(f), r)
    }
}
fn map_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(f) = required_callback(s, a.get(0), "map") {
        operation(s, a.this(), ObservableOperation::Map(f), r)
    }
}
fn switch_map(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(f) = required_callback(s, a.get(0), "switchMap") {
        operation(s, a.this(), ObservableOperation::SwitchMap(f), r)
    }
}
fn drop_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let count = a.get(0).integer_value(s).unwrap_or(0).max(0) as u64;
    operation(s, a.this(), ObservableOperation::Drop(count), r)
}
fn take_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let count = a.get(0).integer_value(s).unwrap_or(0).max(0) as u64;
    operation(s, a.this(), ObservableOperation::Take(count), r)
}
fn inspect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let observer = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .map(|value| v8::Global::new(s, value));
    operation(s, a.this(), ObservableOperation::Inspect(observer), r)
}
fn take_until(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if source(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let Ok(notifier) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "takeUntil requires an Observable");
        return;
    };
    if source(s, notifier).is_none() {
        crate::webidl::throw_type_error(s, "takeUntil requires an Observable");
        return;
    }
    operation(
        s,
        a.this(),
        ObservableOperation::TakeUntil(v8::Global::new(s, notifier)),
        r,
    )
}

fn collected_or_throw(
    scope: &mut v8::PinScope<'_, '_>,
    this: v8::Local<'_, v8::Object>,
) -> Option<Collector> {
    if source(scope, this).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    } else {
        Some(collect(scope, this))
    }
}
fn resolved(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}
fn rejected(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, value) {
        result.set(promise.into())
    }
}
fn ensure_promise_receiver(
    scope: &mut v8::PinScope<'_, '_>,
    this: v8::Local<'_, v8::Object>,
    method_name: &str,
    result: v8::ReturnValue<'_>,
) -> bool {
    if source(scope, this).is_some() {
        true
    } else {
        crate::webidl::reject_illegal_invocation_promise(scope, "Observable", method_name, result);
        false
    }
}
fn predicate_result(
    scope: &mut v8::PinScope<'_, '_>,
    callback: &v8::Global<v8::Function>,
    value: &v8::Global<v8::Value>,
    index: usize,
) -> bool {
    let value = v8::Local::new(scope, value);
    let index = v8::Integer::new_from_unsigned(scope, index as u32);
    callback_result(scope, callback, &[value, index.into()])
        .is_some_and(|value| value.boolean_value(scope))
}
fn first(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "first", r) {
        return;
    }
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    if let Some(value) = c.values.first() {
        resolved(scope, v8::Local::new(scope, value), r)
    } else {
        let message = v8::String::new(scope, "Observable emitted no values").unwrap();
        rejected(scope, v8::Exception::range_error(scope, message), r)
    }
}
fn last(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "last", r) {
        return;
    }
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    if let Some(value) = c.values.last() {
        resolved(scope, v8::Local::new(scope, value), r)
    } else {
        let message = v8::String::new(scope, "Observable emitted no values").unwrap();
        rejected(scope, v8::Exception::range_error(scope, message), r)
    }
}
fn find(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "find", r) {
        return;
    }
    let Some(callback) = required_callback(scope, a.get(0), "find") else {
        return;
    };
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    let found = c
        .values
        .iter()
        .enumerate()
        .find(|(i, v)| predicate_result(scope, &callback, v, *i));
    match found {
        Some((_, v)) => resolved(scope, v8::Local::new(scope, v), r),
        None => resolved(scope, v8::undefined(scope).into(), r),
    }
}
fn every(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "every", r) {
        return;
    }
    let Some(callback) = required_callback(scope, a.get(0), "every") else {
        return;
    };
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    let value = c
        .values
        .iter()
        .enumerate()
        .all(|(i, v)| predicate_result(scope, &callback, v, i));
    resolved(scope, v8::Boolean::new(scope, value).into(), r)
}
fn some(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "some", r) {
        return;
    }
    let Some(callback) = required_callback(scope, a.get(0), "some") else {
        return;
    };
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    let value = c
        .values
        .iter()
        .enumerate()
        .any(|(i, v)| predicate_result(scope, &callback, v, i));
    resolved(scope, v8::Boolean::new(scope, value).into(), r)
}
fn reduce(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "reduce", r) {
        return;
    }
    let Some(callback) = required_callback(scope, a.get(0), "reduce") else {
        return;
    };
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    let mut iterator = c.values.iter();
    let initial = if a.length() >= 2 {
        a.get(1)
    } else if let Some(first) = iterator.next() {
        v8::Local::new(scope, first)
    } else {
        let message = v8::String::new(scope, "Observable emitted no values").unwrap();
        rejected(scope, v8::Exception::range_error(scope, message), r);
        return;
    };
    let mut accumulator = v8::Global::new(scope, initial);
    for (index, value) in iterator.enumerate() {
        let accumulator_local = v8::Local::new(scope, &accumulator);
        let value = v8::Local::new(scope, value);
        let i = v8::Integer::new_from_unsigned(scope, index as u32);
        let Some(next) = callback_result(scope, &callback, &[accumulator_local, value, i.into()])
        else {
            return;
        };
        accumulator = v8::Global::new(scope, next);
    }
    resolved(scope, v8::Local::new(scope, &accumulator), r)
}
fn to_array(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "toArray", r) {
        return;
    }
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    let array = v8::Array::new(scope, c.values.len() as i32);
    for (index, value) in c.values.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, value));
    }
    resolved(scope, array.into(), r)
}
fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !ensure_promise_receiver(scope, a.this(), "forEach", r) {
        return;
    }
    let Some(callback) = required_callback(scope, a.get(0), "forEach") else {
        return;
    };
    let Some(c) = collected_or_throw(scope, a.this()) else {
        return;
    };
    for (index, value) in c.values.iter().enumerate() {
        let value = v8::Local::new(scope, value);
        let i = v8::Integer::new_from_unsigned(scope, index as u32);
        if callback_result(scope, &callback, &[value, i.into()]).is_none() {
            return;
        }
    }
    resolved(scope, v8::undefined(scope).into(), r)
}
fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ObservableStore>() {
        store.constructor.remove(realm_id);
    }
}
