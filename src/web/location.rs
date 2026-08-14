use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LocationStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ::url::Url>,
    ancestor_origins: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LocationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Location", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<LocationStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Location",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<LocationStore>()
        .ok_or_else(|| "Location state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Location': Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    href: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Location".to_owned());
    }
    let url = ::url::Url::parse(href)
        .or_else(|_| ::url::Url::parse("https://example.com/"))
        .map_err(|error| error.to_string())?;
    let ancestor_origins = super::dom_string_list::create(scope, Vec::new())?;
    let identity = object.get_identity_hash().get();
    let ancestor_origins_global = v8::Global::new(scope, ancestor_origins);
    scope
        .get_slot_mut::<LocationStore>()
        .ok_or_else(|| "Location state was not prepared".to_owned())?
        .records
        .insert(identity, url);
    scope
        .get_slot_mut::<LocationStore>()
        .ok_or_else(|| "Location state was not prepared".to_owned())?
        .ancestor_origins
        .insert(identity, ancestor_origins_global);
    define_value_of(scope, object)?;
    define_ancestor_origins(scope, object)?;
    define_location_accessor(scope, object, "href", true)?;
    define_location_accessor(scope, object, "origin", false)?;
    define_location_accessor(scope, object, "protocol", true)?;
    define_location_accessor(scope, object, "host", true)?;
    define_location_accessor(scope, object, "hostname", true)?;
    define_location_accessor(scope, object, "port", true)?;
    define_location_accessor(scope, object, "pathname", true)?;
    define_location_accessor(scope, object, "search", true)?;
    define_location_accessor(scope, object, "hash", true)?;
    define_method(scope, object, "assign", 1, assign)?;
    define_method(scope, object, "reload", 0, reload)?;
    define_method(scope, object, "replace", 1, replace)?;
    define_method(scope, object, "toString", 0, to_string)?;
    let to_primitive = v8::Symbol::get_to_primitive(scope);
    let undefined = v8::undefined(scope);
    if object.define_own_property(
        scope,
        to_primitive.into(),
        undefined.into(),
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) != Some(true)
    {
        return Err("cannot define Location Symbol.toPrimitive".to_owned());
    }
    Ok(object)
}

fn define_value_of(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "valueOf",
        0,
        v8::ConstructorBehavior::Throw,
        value_of,
    )?;
    let key = crate::webidl::string(scope, "valueOf")?;
    if object.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::READ_ONLY
            | v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define Location.valueOf".to_owned())
    }
}

fn define_ancestor_origins(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get ancestorOrigins",
        0,
        v8::ConstructorBehavior::Throw,
        get_ancestor_origins,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(false);
    let key = crate::webidl::string(scope, "ancestorOrigins")?;
    if object.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define Location.ancestorOrigins".to_owned())
    }
}

fn define_location_accessor(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    writable: bool,
) -> Result<(), String> {
    let data = crate::webidl::string(scope, name)?;
    let getter = crate::webidl::create_function_with_data(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        get_component,
        data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, object.into()) {
        crate::trace::relabel_native_function(scope, getter, &format!("{owner}.get {name}"));
    }
    let setter: v8::Local<v8::Value> = if writable {
        let data = crate::webidl::string(scope, name)?;
        let setter = crate::webidl::create_function_with_data(
            scope,
            &format!("set {name}"),
            1,
            v8::ConstructorBehavior::Throw,
            set_component,
            data.into(),
        )?;
        if let Some(owner) = crate::trace::native_label_for_value(scope, object.into()) {
            crate::trace::relabel_native_function(scope, setter, &format!("{owner}.set {name}"));
        }
        setter.into()
    } else {
        v8::undefined(scope).into()
    };
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(false);
    let key = crate::webidl::string(scope, name)?;
    if object.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define Location.{name}"))
    }
}

fn define_method(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    length: i32,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        name,
        length,
        v8::ConstructorBehavior::Throw,
        callback,
    )?;
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define Location.{name}"))
    }
}

fn value_of(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(arguments.this().into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_ancestor_origins(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<LocationStore>()
        .and_then(|store| {
            store
                .ancestor_origins
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<::url::Url> {
    scope
        .get_slot::<LocationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn current_url(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<::url::Url> {
    record(scope, object)
}

pub(crate) fn replace_url(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    url: ::url::Url,
) -> bool {
    let Some(stored) = scope
        .get_slot_mut::<LocationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    *stored = url;
    true
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(url) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let component = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    let value = match component.as_str() {
        "href" => url.as_str().to_owned(),
        "origin" => url.origin().ascii_serialization(),
        "protocol" => format!("{}:", url.scheme()),
        "host" => {
            let host = url.host_str().unwrap_or_default();
            url.port()
                .map(|port| format!("{host}:{port}"))
                .unwrap_or_else(|| host.to_owned())
        }
        "hostname" => url.host_str().unwrap_or_default().to_owned(),
        "port" => url.port().map(|port| port.to_string()).unwrap_or_default(),
        "pathname" => url.path().to_owned(),
        "search" => url
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default(),
        "hash" => url
            .fragment()
            .map(|fragment| format!("#{fragment}"))
            .unwrap_or_default(),
        _ => String::new(),
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(mut url) = scope
        .get_slot::<LocationStore>()
        .and_then(|store| store.records.get(&id))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let component = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let accepted = match component.as_str() {
        "href" => resolve_url(&url, &value).map(|next| url = next).is_some(),
        "protocol" => url.set_scheme(value.trim_end_matches(':')).is_ok(),
        "host" => set_host(&mut url, &value),
        "hostname" => url.set_host(Some(&value)).is_ok(),
        "port" => {
            let port = if value.is_empty() {
                None
            } else {
                value.parse::<u16>().ok()
            };
            url.set_port(port).is_ok()
        }
        "pathname" => {
            url.set_path(&value);
            true
        }
        "search" => {
            url.set_query((!value.is_empty()).then_some(value.trim_start_matches('?')));
            true
        }
        "hash" => {
            url.set_fragment((!value.is_empty()).then_some(value.trim_start_matches('#')));
            true
        }
        _ => false,
    };
    if accepted {
        let target = url.as_str().to_owned();
        if let Some(stored) = scope
            .get_slot_mut::<LocationStore>()
            .and_then(|store| store.records.get_mut(&id))
        {
            *stored = url;
        }
        if let Some(Err(message)) = super::html_i_frame_element::navigate_for_location_object(
            scope,
            arguments.this(),
            target,
        ) {
            crate::webidl::throw_type_error(scope, &message);
        }
    }
}

fn assign(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    navigate(scope, arguments.this(), arguments.get(0));
}

fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    navigate(scope, arguments.this(), arguments.get(0));
}

fn reload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, url.as_str()) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn navigate(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
) {
    let id = object.get_identity_hash().get();
    let Some(current) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let target = crate::webidl::value_to_string(scope, value);
    if let Some(target) = resolve_url(&current, &target) {
        let href = target.as_str().to_owned();
        if let Some(stored) = scope
            .get_slot_mut::<LocationStore>()
            .and_then(|store| store.records.get_mut(&id))
        {
            *stored = target;
        }
        if let Some(Err(message)) =
            super::html_i_frame_element::navigate_for_location_object(scope, object, href)
        {
            crate::webidl::throw_type_error(scope, &message);
        }
    }
}

fn resolve_url(base: &::url::Url, value: &str) -> Option<::url::Url> {
    ::url::Url::parse(value).or_else(|_| base.join(value)).ok()
}

fn set_host(url: &mut ::url::Url, value: &str) -> bool {
    let mut parts = value.rsplitn(2, ':');
    let last = parts.next().unwrap_or_default();
    let first = parts.next();
    if let Some(host) = first
        && let Ok(port) = last.parse::<u16>()
    {
        return url.set_host(Some(host)).is_ok() && url.set_port(Some(port)).is_ok();
    }
    url.set_host(Some(value)).is_ok()
}
