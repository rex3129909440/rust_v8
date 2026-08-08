use std::collections::HashMap;

#[derive(Clone)]
struct NotificationRecord {
    onclick: Option<v8::Global<v8::Value>>,
    onshow: Option<v8::Global<v8::Value>>,
    onerror: Option<v8::Global<v8::Value>>,
    onclose: Option<v8::Global<v8::Value>>,
    title: String,
    dir: String,
    lang: String,
    body: String,
    tag: String,
    icon: String,
    badge: String,
    vibrate: Vec<u32>,
    timestamp: f64,
    renotify: bool,
    silent: Option<bool>,
    require_interaction: bool,
    data: v8::Global<v8::Value>,
    actions: v8::Global<v8::Array>,
    image: String,
    scenario: String,
    closed: bool,
}

#[derive(Default)]
pub(crate) struct NotificationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NotificationRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(NotificationStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "Notification", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<NotificationStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "Notification",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "onclick", get_onclick, set_onclick)?;
    crate::webidl::define_accessor(s, p, "onshow", get_onshow, set_onshow)?;
    crate::webidl::define_accessor(s, p, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(s, p, "onclose", get_onclose, set_onclose)?;
    crate::webidl::define_readonly_accessor(s, p, "title", get_title)?;
    crate::webidl::define_readonly_accessor(s, p, "dir", get_dir)?;
    crate::webidl::define_readonly_accessor(s, p, "lang", get_lang)?;
    crate::webidl::define_readonly_accessor(s, p, "body", get_body)?;
    crate::webidl::define_readonly_accessor(s, p, "tag", get_tag)?;
    crate::webidl::define_readonly_accessor(s, p, "icon", get_icon)?;
    crate::webidl::define_readonly_accessor(s, p, "badge", get_badge)?;
    crate::webidl::define_readonly_accessor(s, p, "vibrate", get_vibrate)?;
    crate::webidl::define_readonly_accessor(s, p, "timestamp", get_timestamp)?;
    crate::webidl::define_readonly_accessor(s, p, "renotify", get_renotify)?;
    crate::webidl::define_readonly_accessor(s, p, "silent", get_silent)?;
    crate::webidl::define_readonly_accessor(s, p, "requireInteraction", get_require_interaction)?;
    crate::webidl::define_readonly_accessor(s, p, "data", get_data)?;
    crate::webidl::define_readonly_accessor(s, p, "actions", get_actions)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_readonly_accessor(s, p, "image", get_image)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_readonly_accessor(s, p, "scenario", get_scenario)?;
    define_static_accessor(s, c.into(), "permission", get_permission)?;
    define_static_accessor(s, c.into(), "maxActions", get_max_actions)?;
    crate::webidl::define_method(s, c.into(), "requestPermission", 0, request_permission)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<NotificationStore>()
        .ok_or_else(|| "Notification state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'Notification': 1 argument required, but only 0 present.",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let data = property(s, init, "data").unwrap_or_else(|| v8::null(s).into());
    let actions = array_property(s, init, "actions").unwrap_or_else(|| v8::Array::new(s, 0));
    let record = NotificationRecord {
        onclick: None,
        onshow: None,
        onerror: None,
        onclose: None,
        title: crate::webidl::value_to_string(s, a.get(0)),
        dir: string_property(s, init, "dir", "auto"),
        lang: string_property(s, init, "lang", ""),
        body: string_property(s, init, "body", ""),
        tag: string_property(s, init, "tag", ""),
        icon: string_property(s, init, "icon", ""),
        badge: string_property(s, init, "badge", ""),
        vibrate: read_vibrate(s, init),
        timestamp: number_property(s, init, "timestamp")
            .unwrap_or_else(|| crate::determinism::date_epoch_milliseconds(s)),
        renotify: bool_property(s, init, "renotify"),
        silent: property(s, init, "silent")
            .filter(|v| !v.is_null())
            .map(|v| v.boolean_value(s)),
        require_interaction: bool_property(s, init, "requireInteraction"),
        data: v8::Global::new(s, data),
        actions: v8::Global::new(s, actions),
        image: string_property(s, init, "image", ""),
        scenario: string_property(s, init, "scenario", "default"),
        closed: false,
    };
    super::event_target::attach(s, a.this());
    s.get_slot_mut::<NotificationStore>()
        .expect("Notification state")
        .records
        .insert(a.this().get_identity_hash().get(), record);
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<NotificationRecord> {
    s.get_slot::<NotificationStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn update(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    f: impl FnOnce(&mut NotificationRecord),
) {
    if let Some(x) = s
        .get_slot_mut::<NotificationStore>()
        .and_then(|x| x.records.get_mut(&o.get_identity_hash().get()))
    {
        f(x)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn handler_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    f: impl FnOnce(NotificationRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, f(x), r)
}
fn handler_set(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    f: impl FnOnce(&mut NotificationRecord, Option<v8::Global<v8::Value>>),
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    update(s, a.this(), |x| f(x, h))
}
macro_rules! handlers {
    ($get:ident,$set:ident,$field:ident) => {
        fn $get(
            s: &mut v8::PinScope<'_, '_>,
            a: v8::FunctionCallbackArguments<'_>,
            r: v8::ReturnValue<'_>,
        ) {
            handler_get(s, a, r, |x| x.$field)
        }
        fn $set(
            s: &mut v8::PinScope<'_, '_>,
            a: v8::FunctionCallbackArguments<'_>,
            _: v8::ReturnValue<'_>,
        ) {
            handler_set(s, a, |x, h| x.$field = h)
        }
    };
}
handlers!(get_onclick, set_onclick, onclick);
handlers!(get_onshow, set_onshow, onshow);
handlers!(get_onerror, set_onerror, onerror);
handlers!(get_onclose, set_onclose, onclose);
fn text_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(&NotificationRecord) -> &str,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(s, f(&x)) {
        r.set(v.into())
    }
}
macro_rules! texts {
    ($name:ident,$field:ident) => {
        fn $name(
            s: &mut v8::PinScope<'_, '_>,
            a: v8::FunctionCallbackArguments<'_>,
            r: v8::ReturnValue<'_>,
        ) {
            text_get(s, a, r, |x| &x.$field)
        }
    };
}
texts!(get_title, title);
texts!(get_dir, dir);
texts!(get_lang, lang);
texts!(get_body, body);
texts!(get_tag, tag);
texts!(get_icon, icon);
texts!(get_badge, badge);
texts!(get_image, image);
texts!(get_scenario, scenario);
fn get_vibrate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let v = v8::Array::new(s, x.vibrate.len() as i32);
    for (i, n) in x.vibrate.iter().enumerate() {
        let n = v8::Integer::new_from_unsigned(s, *n);
        let _ = v.set_index(s, i as u32, n.into());
    }
    r.set(v.into())
}
fn get_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    r.set(v8::Number::new(s, x.timestamp).into())
}
fn bool_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(&NotificationRecord) -> bool,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    r.set(v8::Boolean::new(s, f(&x)).into())
}
fn get_renotify(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    bool_get(s, a, r, |x| x.renotify)
}
fn get_require_interaction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    bool_get(s, a, r, |x| x.require_interaction)
}
fn get_silent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    match x.silent {
        Some(v) => r.set(v8::Boolean::new(s, v).into()),
        None => r.set(v8::null(s).into()),
    }
}
fn get_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.data))
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_actions(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.actions).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let already = record(s, a.this()).is_some_and(|x| x.closed);
    if already {
        return;
    }
    let handler = record(s, a.this()).and_then(|record| record.onclose);
    update(s, a.this(), |x| x.closed = true);
    if let Ok(e) = super::event::create(s, "close") {
        super::event_target::dispatch(s, a.this(), e);
        if let Some(handler) = handler
            && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(s, &handler))
        {
            let _ = handler.call(s, a.this().into(), &[e.into()]);
        }
    }
}
fn get_permission(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let permission = notification_permission(s);
    return_static_text(s, &permission, r)
}
fn get_max_actions(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::Integer::new(s, 2).into())
}
fn request_permission(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let permission = notification_permission(s);
    let Some(value) = v8::String::new(s, &permission) else {
        return;
    };
    if let Ok(p) = super::writable_stream::resolved_promise(s, value.into()) {
        r.set(p.into())
    }
    if let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) {
        let u = v8::undefined(s);
        let _ = callback.call(s, u.into(), &[value.into()]);
    }
}
fn notification_permission(s: &v8::PinScope<'_, '_>) -> String {
    match crate::fingerprint::edge(s)
        .permissions
        .notifications
        .as_str()
    {
        "prompt" => "default".to_owned(),
        value => value.to_owned(),
    }
}
fn return_static_text(s: &mut v8::PinScope<'_, '_>, v: &str, mut r: v8::ReturnValue<'_>) {
    if let Some(v) = v8::String::new(s, v) {
        r.set(v.into())
    }
}
fn define_static_accessor(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
    g: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let g = crate::webidl::create_function(
        s,
        &format!("get {n}"),
        0,
        v8::ConstructorBehavior::Throw,
        g,
    )?;
    let u = v8::undefined(s);
    let mut d = v8::PropertyDescriptor::new_from_get_set(g.into(), u.into());
    d.set_enumerable(true);
    d.set_configurable(true);
    let k = crate::webidl::string(s, n)?;
    if o.define_property(s, k.into(), &d) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define Notification.{n}"))
    }
}
fn property<'s>(
    s: &v8::PinScope<'s, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    n: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let k = v8::String::new(s, n)?;
    o?.get(s, k.into())
}
fn string_property(
    s: &v8::PinScope<'_, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    n: &str,
    d: &str,
) -> String {
    property(s, o, n)
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_else(|| d.to_owned())
}
fn number_property(
    s: &v8::PinScope<'_, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    n: &str,
) -> Option<f64> {
    property(s, o, n)?.number_value(s)
}
fn bool_property(s: &v8::PinScope<'_, '_>, o: Option<v8::Local<'_, v8::Object>>, n: &str) -> bool {
    property(s, o, n).is_some_and(|v| v.boolean_value(s))
}
fn array_property<'s>(
    s: &v8::PinScope<'s, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    n: &str,
) -> Option<v8::Local<'s, v8::Array>> {
    v8::Local::<v8::Array>::try_from(property(s, o, n)?).ok()
}
fn read_vibrate(s: &v8::PinScope<'_, '_>, o: Option<v8::Local<'_, v8::Object>>) -> Vec<u32> {
    let Some(a) = array_property(s, o, "vibrate") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for i in 0..a.length() {
        if let Some(v) = a.get_index(s, i).and_then(|v| v.uint32_value(s)) {
            out.push(v)
        }
    }
    out
}
pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<NotificationStore>() {
        store.constructor.remove(realm_id);
    }
}
