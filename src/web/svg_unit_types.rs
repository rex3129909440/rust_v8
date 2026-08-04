#[derive(Default)]
pub(crate) struct SvgUnitTypesStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgUnitTypesStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let existing = scope
        .get_slot::<SvgUnitTypesStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    let constructor = if let Some(existing) = existing {
        v8::Local::new(scope, &existing)
    } else {
        let constructor = crate::webidl::create_function(
            scope,
            "SVGUnitTypes",
            0,
            v8::ConstructorBehavior::Allow,
            illegal_constructor,
        )?;
        let prototype = crate::webidl::prototype(scope, constructor)?;
        crate::webidl::reset_constructor_order(scope, prototype)?;
        define_constants(scope, prototype)?;
        crate::webidl::finish_constructor(scope, prototype, constructor)?;
        define_constants(scope, constructor.into())?;
        let realm_id = crate::webidl::realm_id(scope);
        let realm_constructor = v8::Global::new(scope, constructor);
        scope
            .get_slot_mut::<SvgUnitTypesStore>()
            .ok_or_else(|| "SVGUnitTypes state was not prepared".to_owned())?
            .constructor
            .insert(realm_id, realm_constructor);
        constructor
    };
    crate::webidl::define_global(scope, "SVGUnitTypes", constructor.into())
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "SVG_UNIT_TYPE_UNKNOWN", 0)?;
    crate::webidl::define_constant(scope, object, "SVG_UNIT_TYPE_USERSPACEONUSE", 1)?;
    crate::webidl::define_constant(scope, object, "SVG_UNIT_TYPE_OBJECTBOUNDINGBOX", 2)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGUnitTypes': Illegal constructor",
    );
}
