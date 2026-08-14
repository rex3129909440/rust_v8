use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Path2dStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<PathCommand>>,
}

#[derive(Clone)]
enum PathCommand {
    Source(String),
    MoveTo(f64, f64),
    LineTo(f64, f64),
    Rect(f64, f64, f64, f64),
    RoundRect(f64, f64, f64, f64, Vec<f64>),
    Arc(f64, f64, f64, f64, f64, bool),
    ArcTo(f64, f64, f64, f64, f64),
    BezierCurveTo(f64, f64, f64, f64, f64, f64),
    QuadraticCurveTo(f64, f64, f64, f64),
    Ellipse(f64, f64, f64, f64, f64, f64, f64, bool),
    Close,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(Path2dStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Path2D", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<Path2dStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Path2D",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "addPath", 1, add_path)?;
    crate::webidl::define_method(scope, prototype, "roundRect", 4, round_rect)?;
    crate::webidl::define_method(scope, prototype, "arc", 5, arc)?;
    crate::webidl::define_method(scope, prototype, "arcTo", 5, arc_to)?;
    crate::webidl::define_method(scope, prototype, "bezierCurveTo", 6, bezier_curve_to)?;
    crate::webidl::define_method(scope, prototype, "closePath", 0, close_path)?;
    crate::webidl::define_method(scope, prototype, "ellipse", 7, ellipse)?;
    crate::webidl::define_method(scope, prototype, "lineTo", 2, line_to)?;
    crate::webidl::define_method(scope, prototype, "moveTo", 2, move_to)?;
    crate::webidl::define_method(scope, prototype, "quadraticCurveTo", 4, quadratic_curve_to)?;
    crate::webidl::define_method(scope, prototype, "rect", 4, rect)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<Path2dStore>()
        .ok_or_else(|| "Path2D state was not prepared".to_owned())?
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
            "Failed to construct 'Path2D': Please use the 'new' operator",
        );
        return;
    }
    let source = arguments.get(0);
    let commands = if source.is_undefined() {
        Vec::new()
    } else if let Ok(object) = v8::Local::<v8::Object>::try_from(source) {
        match record(scope, object) {
            Some(commands) => commands,
            None => vec![PathCommand::Source(crate::webidl::value_to_string(
                scope, source,
            ))],
        }
    } else {
        vec![PathCommand::Source(crate::webidl::value_to_string(
            scope, source,
        ))]
    };
    scope
        .get_slot_mut::<Path2dStore>()
        .expect("Path2D state")
        .records
        .insert(arguments.this().get_identity_hash().get(), commands);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<PathCommand>> {
    scope
        .get_slot::<Path2dStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn append(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    command: PathCommand,
) {
    if let Some(commands) = scope
        .get_slot_mut::<Path2dStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        commands.push(command);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn number(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
) -> f64 {
    arguments.get(index).number_value(scope).unwrap_or(f64::NAN)
}

fn add_path(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(path) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "addPath requires a Path2D");
        return;
    };
    let Some(commands) = record(scope, path) else {
        crate::webidl::throw_type_error(scope, "addPath requires a Path2D");
        return;
    };
    if let Some(target) = scope.get_slot_mut::<Path2dStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        target.extend(commands);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn round_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let radii = if arguments.get(4).is_undefined() {
        vec![0.0]
    } else {
        vec![number(scope, &arguments, 4)]
    };
    append(
        scope,
        arguments.this(),
        PathCommand::RoundRect(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            number(scope, &arguments, 2),
            number(scope, &arguments, 3),
            radii,
        ),
    );
}

fn arc(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let radius = number(scope, &arguments, 2);
    if radius < 0.0 {
        throw_index_size(scope, "The radius provided is negative");
        return;
    }
    append(
        scope,
        arguments.this(),
        PathCommand::Arc(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            radius,
            number(scope, &arguments, 3),
            number(scope, &arguments, 4),
            arguments.get(5).boolean_value(scope),
        ),
    );
}

fn arc_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let radius = number(scope, &arguments, 4);
    if radius < 0.0 {
        throw_index_size(scope, "The radius provided is negative");
        return;
    }
    append(
        scope,
        arguments.this(),
        PathCommand::ArcTo(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            number(scope, &arguments, 2),
            number(scope, &arguments, 3),
            radius,
        ),
    );
}

fn bezier_curve_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    append(
        scope,
        arguments.this(),
        PathCommand::BezierCurveTo(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            number(scope, &arguments, 2),
            number(scope, &arguments, 3),
            number(scope, &arguments, 4),
            number(scope, &arguments, 5),
        ),
    );
}
fn close_path(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    append(scope, arguments.this(), PathCommand::Close);
}
fn line_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    append(
        scope,
        arguments.this(),
        PathCommand::LineTo(number(scope, &arguments, 0), number(scope, &arguments, 1)),
    );
}
fn move_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    append(
        scope,
        arguments.this(),
        PathCommand::MoveTo(number(scope, &arguments, 0), number(scope, &arguments, 1)),
    );
}
fn quadratic_curve_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    append(
        scope,
        arguments.this(),
        PathCommand::QuadraticCurveTo(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            number(scope, &arguments, 2),
            number(scope, &arguments, 3),
        ),
    );
}
fn rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    append(
        scope,
        arguments.this(),
        PathCommand::Rect(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            number(scope, &arguments, 2),
            number(scope, &arguments, 3),
        ),
    );
}

fn ellipse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let radius_x = number(scope, &arguments, 2);
    let radius_y = number(scope, &arguments, 3);
    if radius_x < 0.0 || radius_y < 0.0 {
        throw_index_size(scope, "The radius provided is negative");
        return;
    }
    append(
        scope,
        arguments.this(),
        PathCommand::Ellipse(
            number(scope, &arguments, 0),
            number(scope, &arguments, 1),
            radius_x,
            radius_y,
            number(scope, &arguments, 4),
            number(scope, &arguments, 5),
            number(scope, &arguments, 6),
            arguments.get(7).boolean_value(scope),
        ),
    );
}

fn throw_index_size(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), "IndexSizeError".to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<Path2dStore>() {
        store.constructor.remove(realm_id);
    }
}
