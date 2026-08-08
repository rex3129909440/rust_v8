use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ConsoleEdgeStore {
    memory: HashMap<i32, v8::Global<v8::Object>>,
    tasks: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ConsoleEdgeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let console_key = crate::webidl::string(scope, "console")?;
    let console = global
        .get(scope, console_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "V8 console is unavailable".to_owned())?;

    install_stdout_method(scope, console, "debug", console_debug)?;
    install_stdout_method(scope, console, "info", console_info)?;
    install_stdout_method(scope, console, "log", console_log)?;
    install_stdout_method(scope, console, "warn", console_warn)?;
    install_stdout_method(scope, console, "error", console_error)?;
    install_stdout_method(scope, console, "dir", console_dir)?;
    install_stdout_method(scope, console, "dirxml", console_dirxml)?;
    install_stdout_method(scope, console, "table", console_table)?;
    install_stdout_method(scope, console, "trace", console_trace)?;
    install_method(scope, console, "group", console_group)?;
    install_method(scope, console, "groupCollapsed", console_group)?;
    install_method(scope, console, "assert", console_assert)?;
    install_method(scope, console, "count", console_label)?;
    install_method(scope, console, "countReset", console_label)?;
    install_method(scope, console, "time", console_label)?;
    install_method(scope, console, "timeLog", console_label)?;
    install_method(scope, console, "timeEnd", console_label)?;
    install_method(scope, console, "timeStamp", console_label)?;
    install_method(scope, console, "profile", console_label)?;
    install_method(scope, console, "profileEnd", console_label)?;

    let create_task = crate::webidl::create_function(
        scope,
        "createTask",
        0,
        v8::ConstructorBehavior::Throw,
        create_task,
    )?;
    let create_task_key = crate::webidl::string(scope, "createTask")?;
    if console.define_own_property(
        scope,
        create_task_key.into(),
        create_task.into(),
        v8::PropertyAttribute::NONE,
    ) != Some(true)
    {
        return Err("cannot define console.createTask".to_owned());
    }

    let memory = create_memory_info(scope)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, memory);
    scope
        .get_slot_mut::<ConsoleEdgeStore>()
        .ok_or_else(|| "console Edge state was not prepared".to_owned())?
        .memory
        .insert(realm_id, stored);
    let getter =
        crate::webidl::create_function(scope, "", 0, v8::ConstructorBehavior::Throw, get_memory)?;
    let setter =
        crate::webidl::create_function(scope, "", 0, v8::ConstructorBehavior::Throw, set_memory)?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let memory_key = crate::webidl::string(scope, "memory")?;
    if console.define_property(scope, memory_key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define console.memory".to_owned())
    }
}

fn install_stdout_method(
    scope: &mut v8::PinScope<'_, '_>,
    console: v8::Local<'_, v8::Object>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, name, 0, v8::ConstructorBehavior::Throw, callback)?;
    let key = crate::webidl::string(scope, name)?;
    if console.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot install console.{name} stdout capture"))
    }
}

fn install_method(
    scope: &mut v8::PinScope<'_, '_>,
    console: v8::Local<'_, v8::Object>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, name, 0, v8::ConstructorBehavior::Throw, callback)?;
    let key = crate::webidl::string(scope, name)?;
    if console.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot install console.{name}"))
    }
}

fn console_debug<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Debug, &arguments);
}

fn console_info<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Info, &arguments);
}

fn console_log<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Log, &arguments);
}

fn console_warn<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Warn, &arguments);
}

fn console_error<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Error, &arguments);
}

fn console_dir<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Dir, &arguments);
}

fn console_dirxml<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::DirXml, &arguments);
}

fn console_table<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Table, &arguments);
}

fn console_trace<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::record(scope, crate::ConsoleLevel::Trace, &arguments);
}

fn console_group<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    crate::console_capture::observe_arguments(scope, &arguments, 0);
}

fn console_assert<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    _: v8::ReturnValue<'s>,
) {
    if !arguments.get(0).boolean_value(scope) {
        crate::console_capture::observe_arguments(scope, &arguments, 1);
    }
}

fn console_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let label = arguments.get(0);
    if !label.is_undefined() {
        let _ = label.to_string(scope);
    }
}

fn create_memory_info<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let prototype = v8::Object::new(scope);
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "totalJSHeapSize",
        total_js_heap_size,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "usedJSHeapSize", used_js_heap_size)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "jsHeapSizeLimit",
        js_heap_size_limit,
    )?;
    let tag = v8::Symbol::get_to_string_tag(scope);
    let tag_value = crate::webidl::string(scope, "MemoryInfo")?;
    if prototype.define_own_property(
        scope,
        tag.into(),
        tag_value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define MemoryInfo toStringTag".to_owned());
    }
    let memory = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, memory, prototype.into()) != Some(true) {
        return Err("cannot create MemoryInfo".to_owned());
    }
    Ok(memory)
}

fn create_task(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = arguments.get(0);
    let valid_name = v8::Local::<v8::String>::try_from(name)
        .ok()
        .is_some_and(|name| name.length() > 0);
    if !valid_name {
        throw_console_error(scope, "First argument must be a non-empty string.");
        return;
    }
    let task = v8::Object::new(scope);
    let run = match crate::webidl::create_function(
        scope,
        "run",
        0,
        v8::ConstructorBehavior::Allow,
        run_task,
    ) {
        Ok(run) => run,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let key = v8::String::new(scope, "run").expect("short console task key");
    let _ = task.define_own_property(scope, key.into(), run.into(), v8::PropertyAttribute::NONE);
    let realm_id = crate::webidl::realm_id(scope);
    let stored_task = v8::Global::new(scope, task);
    if let Some(store) = scope.get_slot_mut::<ConsoleEdgeStore>() {
        store.tasks.entry(realm_id).or_default().push(stored_task);
    }
    result.set(task.into());
}

fn run_task(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        throw_console_error(scope, "First argument must be a function.");
        return;
    };
    let realm_id = crate::webidl::realm_id(scope);
    let valid_receiver = scope
        .get_slot::<ConsoleEdgeStore>()
        .and_then(|store| store.tasks.get(&realm_id))
        .is_some_and(|tasks| {
            tasks
                .iter()
                .any(|task| v8::Local::new(scope, task).strict_equals(arguments.this().into()))
        });
    if !valid_receiver {
        throw_console_error(scope, "'run' called with illegal receiver.");
        return;
    }
    let receiver = v8::undefined(scope);
    if let Some(value) = callback.call(scope, receiver.into(), &[]) {
        result.set(value)
    }
}

fn throw_console_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    let Some(message) = v8::String::new(scope, message) else {
        return;
    };
    let exception = v8::Exception::error(scope, message);
    scope.throw_exception(exception);
}

fn get_memory(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(memory) = scope
        .get_slot::<ConsoleEdgeStore>()
        .and_then(|store| store.memory.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &memory).into())
    }
}

fn set_memory(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let _ignored_assignment = arguments.get(0).to_string(scope);
    result.set(v8::undefined(scope).into());
}

fn total_js_heap_size(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = crate::fingerprint::edge(scope)
        .memory
        .console_total_js_heap_size as f64;
    result.set(v8::Number::new(scope, value).into())
}

fn used_js_heap_size(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = crate::fingerprint::edge(scope)
        .memory
        .console_used_js_heap_size as f64;
    result.set(v8::Number::new(scope, value).into())
}

fn js_heap_size_limit(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = crate::fingerprint::edge(scope)
        .memory
        .console_js_heap_size_limit as f64;
    result.set(v8::Number::new(scope, value).into())
}
