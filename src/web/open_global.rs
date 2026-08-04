use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct OpenWindowStore {
    records: HashMap<i32, OpenWindowRecord>,
}

#[derive(Clone)]
struct OpenWindowRecord {
    name: String,
    url: String,
    closed: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OpenWindowStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "open", 0, v8::ConstructorBehavior::Throw, open)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "open")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.open".to_owned())
    }
}

fn open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let url = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let target = if arguments.get(1).is_undefined() {
        "_blank".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    if matches!(target.as_str(), "_self" | "_top" | "_parent") {
        result.set(scope.get_current_context().global(scope).into());
        return;
    }
    match create_child(scope, target, url) {
        Ok(child) => result.set(child.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_child<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    url: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = super::window::ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let child = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, child, prototype.into()) != Some(true) {
        return Err("cannot create child Window".to_owned());
    }
    super::event_target::attach(scope, child);
    scope
        .get_slot_mut::<OpenWindowStore>()
        .ok_or_else(|| "child window state was not prepared".to_owned())?
        .records
        .insert(
            child.get_identity_hash().get(),
            OpenWindowRecord {
                name,
                url,
                closed: false,
            },
        );
    define_readonly_accessor(scope, child, "closed", get_child_closed)?;
    define_accessor(scope, child, "name", get_child_name, set_child_name)?;
    define_readonly_accessor(scope, child, "location", get_child_location)?;
    let close = crate::webidl::create_function(
        scope,
        "close",
        0,
        v8::ConstructorBehavior::Throw,
        close_child,
    )?;
    define_data(scope, child, "close", close.into())?;
    let global = scope.get_current_context().global(scope);
    define_data(scope, child, "opener", global.into())?;
    define_data(scope, child, "window", child.into())?;
    define_data(scope, child, "self", child.into())?;
    define_data(scope, child, "top", child.into())?;
    define_data(scope, child, "parent", child.into())?;
    Ok(child)
}

fn define_accessor(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    getter_callback: impl v8::MapFnTo<v8::FunctionCallback>,
    setter_callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        getter_callback,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        setter_callback,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, name)?;
    if object.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define child Window.{name}"))
    }
}

fn define_readonly_accessor(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    getter_callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        getter_callback,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, name)?;
    if object.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define child Window.{name}"))
    }
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE)
        == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define child Window.{name}"))
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<OpenWindowRecord> {
    scope
        .get_slot::<OpenWindowStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_child_closed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.closed).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_child_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(name) = v8::String::new(scope, &record.name) {
            result.set(name.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_child_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<OpenWindowStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.name = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_child_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let value = if record.url.is_empty() {
            "about:blank"
        } else {
            &record.url
        };
        if let Some(url) = v8::String::new(scope, value) {
            result.set(url.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn close_child(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<OpenWindowStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.closed = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
