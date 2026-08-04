use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ProcessingInstructionStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    targets: HashMap<i32, String>,
    attributes: HashMap<i32, Vec<(String, String)>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ProcessingInstructionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ProcessingInstruction", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ProcessingInstructionStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ProcessingInstruction",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::processing_instruction_target_property::define(scope, prototype)?;
    super::processing_instruction_sheet_property::define(scope, prototype)?;
    super::processing_instruction_get_attribute::define(scope, prototype)?;
    super::processing_instruction_get_attribute_names::define(scope, prototype)?;
    super::processing_instruction_has_attribute::define(scope, prototype)?;
    super::processing_instruction_has_attributes::define(scope, prototype)?;
    super::processing_instruction_remove_attribute::define(scope, prototype)?;
    super::processing_instruction_set_attribute::define(scope, prototype)?;
    super::processing_instruction_toggle_attribute::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::character_data::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ProcessingInstructionStore>()
        .ok_or_else(|| "ProcessingInstruction state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ProcessingInstruction': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: String,
    data: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let instruction = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, instruction, prototype.into()) != Some(true) {
        return Err("cannot create ProcessingInstruction".to_owned());
    }
    super::node::attach(scope, instruction, 7, target.clone(), Some(data.clone()));
    super::character_data::attach(scope, instruction, data);
    let identity = instruction.get_identity_hash().get();
    let store = scope
        .get_slot_mut::<ProcessingInstructionStore>()
        .ok_or_else(|| "ProcessingInstruction state was not prepared".to_owned())?;
    store.targets.insert(identity, target);
    store.attributes.insert(identity, Vec::new());
    Ok(instruction)
}

pub(crate) fn target(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    scope
        .get_slot::<ProcessingInstructionStore>()?
        .targets
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn attributes(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, String)>> {
    scope
        .get_slot::<ProcessingInstructionStore>()?
        .attributes
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn write_attributes(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    attributes: &[(String, String)],
) {
    let mut data = String::new();
    for (index, (name, value)) in attributes.iter().enumerate() {
        if index > 0 {
            data.push(' ');
        }
        data.push_str(name);
        data.push_str("=\"");
        data.push_str(value);
        data.push('"');
    }
    let _ = super::character_data::set_data_if_character(scope, object, data);
    if let Some(store) = scope.get_slot_mut::<ProcessingInstructionStore>() {
        store
            .attributes
            .insert(object.get_identity_hash().get(), attributes.to_vec());
    }
}

pub(crate) fn requested_name(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> String {
    crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase()
}

pub(crate) fn validate_attribute_name(scope: &mut v8::PinScope<'_, '_>, name: &str) -> bool {
    if super::document::valid_xml_name(name) {
        true
    } else {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            &format!(
                "Failed to execute 'setAttribute' on 'ProcessingInstruction': Invalid attribute name: {name}"
            ),
        );
        false
    }
}
