use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgAnimationElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) target: Option<v8::Global<v8::Object>>,
    pub(crate) onbegin: Option<v8::Global<v8::Function>>,
    pub(crate) onend: Option<v8::Global<v8::Function>>,
    pub(crate) onrepeat: Option<v8::Global<v8::Function>>,
    pub(crate) required_extensions: v8::Global<v8::Object>,
    pub(crate) system_language: v8::Global<v8::Object>,
    pub(crate) start_time: f64,
    pub(crate) current_time: f64,
    pub(crate) duration: f64,
    pub(crate) active: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgAnimationElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGAnimationElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgAnimationElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGAnimationElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_animation_element_target_element_property::define(scope, prototype)?;
    super::svg_animation_element_onbegin_property::define(scope, prototype)?;
    super::svg_animation_element_onend_property::define(scope, prototype)?;
    super::svg_animation_element_onrepeat_property::define(scope, prototype)?;
    super::svg_animation_element_required_extensions_property::define(scope, prototype)?;
    super::svg_animation_element_system_language_property::define(scope, prototype)?;
    super::svg_animation_element_begin_element::define(scope, prototype)?;
    super::svg_animation_element_begin_element_at::define(scope, prototype)?;
    super::svg_animation_element_end_element::define(scope, prototype)?;
    super::svg_animation_element_end_element_at::define(scope, prototype)?;
    super::svg_animation_element_get_current_time::define(scope, prototype)?;
    super::svg_animation_element_get_simple_duration::define(scope, prototype)?;
    super::svg_animation_element_get_start_time::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgAnimationElementStore>()
        .ok_or_else(|| "SVGAnimationElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = super::svg_element::create_with_constructor(scope, constructor, tag_name, owner)?;
    let required_extensions = super::svg_string_list::create(scope, Vec::new())?;
    let system_language = super::svg_string_list::create(scope, Vec::new())?;
    let record = Record {
        target: None,
        onbegin: None,
        onend: None,
        onrepeat: None,
        required_extensions: v8::Global::new(scope, required_extensions),
        system_language: v8::Global::new(scope, system_language),
        start_time: 0.0,
        current_time: 0.0,
        duration: 0.0,
        active: false,
    };
    scope
        .get_slot_mut::<SvgAnimationElementStore>()
        .ok_or_else(|| "SVGAnimationElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGAnimationElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgAnimationElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(target) = record.target {
        result.set(v8::Local::new(scope, &target).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    select: impl FnOnce(Record) -> Option<v8::Global<v8::Function>>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = select(record) {
        result.set(v8::Local::new(scope, &handler).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn new_handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Function>> {
    v8::Local::<v8::Function>::try_from(value)
        .ok()
        .map(|function| v8::Global::new(scope, function))
}

pub(crate) fn get_onbegin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a.this(), |record| record.onbegin, r);
}
pub(crate) fn get_onend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a.this(), |record| record.onend, r);
}
pub(crate) fn get_onrepeat(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a.this(), |record| record.onrepeat, r);
}

pub(crate) fn set_onbegin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = new_handler(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.onbegin = handler);
}
pub(crate) fn set_onend(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = new_handler(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.onend = handler);
}
pub(crate) fn set_onrepeat(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = new_handler(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.onrepeat = handler);
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(record) = scope
        .get_slot_mut::<SvgAnimationElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_list(
    scope: &v8::PinScope<'_, '_>,
    value: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, value).into());
}

pub(crate) fn get_required_extensions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_list(scope, &record.required_extensions, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_system_language(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_list(scope, &record.system_language, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    event_type: &str,
    handler: Option<v8::Global<v8::Function>>,
) {
    let event = super::event_target::create_event(scope, event_type);
    super::event_target::dispatch(scope, object, event);
    if let Some(handler) = handler {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, object.into(), &[event.into()]);
    }
}

pub(crate) fn begin_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let current_time = container_current_time(scope, arguments.this()).unwrap_or(0.0);
    let handler = record(scope, arguments.this()).and_then(|record| record.onbegin);
    update(scope, arguments.this(), |record| {
        record.active = true;
        record.current_time = current_time;
        record.start_time = current_time;
    });
    fire(scope, arguments.this(), "begin", handler);
    result.set(v8::Boolean::new(scope, true).into());
}

pub(crate) fn begin_element_at(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let offset = arguments.get(0).number_value(scope).unwrap_or(0.0);
    let current_time = container_current_time(scope, arguments.this()).unwrap_or(0.0);
    let handler = record(scope, arguments.this()).and_then(|record| record.onbegin);
    update(scope, arguments.this(), |record| {
        record.active = true;
        record.current_time = current_time;
        record.start_time = current_time + offset;
    });
    fire(scope, arguments.this(), "begin", handler);
    result.set(v8::Boolean::new(scope, true).into());
}

pub(crate) fn end_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| record.onend);
    update(scope, arguments.this(), |record| record.active = false);
    fire(scope, arguments.this(), "end", handler);
    result.set(v8::Boolean::new(scope, true).into());
}

pub(crate) fn end_element_at(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let offset = arguments.get(0).number_value(scope).unwrap_or(0.0);
    let current_time = container_current_time(scope, arguments.this()).unwrap_or(0.0);
    let handler = record(scope, arguments.this()).and_then(|record| record.onend);
    update(scope, arguments.this(), |record| {
        record.current_time = current_time;
        record.duration = (current_time + offset - record.start_time).max(0.0);
        record.active = false;
    });
    fire(scope, arguments.this(), "end", handler);
    result.set(v8::Boolean::new(scope, true).into());
}

pub(crate) fn get_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        let current_time = container_current_time(scope, arguments.this()).unwrap_or(0.0);
        result.set(v8::Number::new(scope, current_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn container_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    let mut candidate = super::node::parent(scope, object);
    while let Some(node) = candidate {
        if super::svg_svg_element::record(scope, node).is_some() {
            return super::svg_svg_element::current_time(scope, node);
        }
        candidate = super::node::parent(scope, node);
    }
    None
}
pub(crate) fn get_duration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.duration).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_start_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.start_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
