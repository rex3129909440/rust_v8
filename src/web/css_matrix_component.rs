use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssMatrixComponentStore {
    constructor: crate::webidl::RealmConstructor,
    matrices: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssMatrixComponentStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSMatrixComponent", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssMatrixComponentStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSMatrixComponent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "matrix", get_matrix, set_matrix)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssMatrixComponentStore>()
        .ok_or_else(|| "CSSMatrixComponent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn is_2d(matrix: &[f64; 16]) -> bool {
    matrix[2] == 0.0
        && matrix[3] == 0.0
        && matrix[6] == 0.0
        && matrix[7] == 0.0
        && matrix[8] == 0.0
        && matrix[9] == 0.0
        && matrix[10] == 1.0
        && matrix[11] == 0.0
        && matrix[14] == 0.0
        && matrix[15] == 1.0
}

fn clone_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<(v8::Global<v8::Object>, bool)> {
    let matrix = super::dom_matrix::matrix_from_value(scope, value).ok()?;
    let two_d = is_2d(&matrix);
    let object = super::dom_matrix::create_from_matrix(scope, matrix).ok()?;
    Some((v8::Global::new(scope, object), two_d))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CSSMatrixComponent requires a matrix");
        return;
    }
    let matrix_value = arguments.get(0);
    let valid_matrix = v8::Local::<v8::Object>::try_from(matrix_value)
        .ok()
        .is_some_and(|object| {
            super::structured_clone::inherits_platform_interface(scope, object, "DOMMatrixReadOnly")
        });
    if !valid_matrix {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSMatrixComponent': parameter 1 is not of type 'DOMMatrixReadOnly'.",
        );
        return;
    }
    let Some((matrix, two_d)) = clone_matrix(scope, matrix_value) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSMatrixComponent': parameter 1 is not of type 'DOMMatrixReadOnly'.",
        );
        return;
    };
    scope
        .get_slot_mut::<CssMatrixComponentStore>()
        .expect("CSSMatrixComponent state")
        .matrices
        .insert(arguments.this().get_identity_hash().get(), matrix);
    super::css_transform_component::attach(scope, arguments.this(), two_d);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<CssMatrixComponentStore>()?
        .matrices
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(matrix) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &matrix).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some((matrix, _)) = clone_matrix(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Invalid matrix");
        return;
    };
    if let Some(current) = scope
        .get_slot_mut::<CssMatrixComponentStore>()
        .and_then(|store| {
            store
                .matrices
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = matrix;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let matrix = record(scope, object)?;
    super::dom_matrix::matrix_snapshot(scope, v8::Local::new(scope, &matrix))
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let matrix = matrix(scope, object)?;
    if is_2d(&matrix) {
        Some(format!(
            "matrix({}, {}, {}, {}, {}, {})",
            matrix[0], matrix[1], matrix[4], matrix[5], matrix[12], matrix[13]
        ))
    } else {
        Some(format!(
            "matrix3d({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
            matrix[0],
            matrix[1],
            matrix[2],
            matrix[3],
            matrix[4],
            matrix[5],
            matrix[6],
            matrix[7],
            matrix[8],
            matrix[9],
            matrix[10],
            matrix[11],
            matrix[12],
            matrix[13],
            matrix[14],
            matrix[15]
        ))
    }
}
