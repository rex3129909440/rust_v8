use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct PeriodicWaveStore {
    constructor: crate::webidl::RealmConstructor,
    context_identities: HashSet<i32>,
    records: HashMap<i32, PeriodicWaveRecord>,
}

#[derive(Clone)]
struct PeriodicWaveRecord {
    real: Vec<f32>,
    imaginary: Vec<f32>,
    disable_normalization: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PeriodicWaveStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PeriodicWave", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PeriodicWaveStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PeriodicWave",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PeriodicWaveStore>()
        .ok_or_else(|| "PeriodicWave state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn register_context(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) {
    if let Some(store) = scope.get_slot_mut::<PeriodicWaveStore>() {
        store
            .context_identities
            .insert(context.get_identity_hash().get());
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PeriodicWave': 1 argument required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    };
    let valid_context = scope.get_slot::<PeriodicWaveStore>().is_some_and(|store| {
        store
            .context_identities
            .contains(&context.get_identity_hash().get())
    });
    if !valid_context {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let real = options
        .and_then(|options| read_sequence(scope, options, "real"))
        .unwrap_or_else(|| vec![0.0, 1.0]);
    let imaginary = options
        .and_then(|options| read_sequence(scope, options, "imag"))
        .unwrap_or_else(|| vec![0.0, 0.0]);
    if real.len() != imaginary.len() || real.len() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "real and imag must have equal lengths of at least two",
        );
        return;
    }
    let disable_normalization = options
        .map(|options| super::event::boolean_property(scope, options, "disableNormalization"))
        .unwrap_or(false);
    scope
        .get_slot_mut::<PeriodicWaveStore>()
        .expect("PeriodicWave state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            PeriodicWaveRecord {
                real,
                imaginary,
                disable_normalization,
            },
        );
    result.set(arguments.this().into());
}

fn read_sequence(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<Vec<f32>> {
    let key = v8::String::new(scope, name)?;
    let value = options.get(scope, key.into())?;
    let sequence = v8::Local::<v8::Object>::try_from(value).ok()?;
    let length_key = v8::String::new(scope, "length")?;
    let length = sequence
        .get(scope, length_key.into())?
        .uint32_value(scope)
        .unwrap_or(0);
    let mut output = Vec::with_capacity(length as usize);
    for index in 0..length {
        let value = sequence
            .get_index(scope, index)
            .and_then(|value| value.number_value(scope))
            .unwrap_or(0.0);
        output.push(value as f32);
    }
    Some(output)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    real: Vec<f32>,
    imaginary: Vec<f32>,
    disable_normalization: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if real.len() != imaginary.len() || real.len() < 2 {
        return Err("real and imaginary coefficients must have equal lengths".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let wave = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, wave, prototype.into()) != Some(true) {
        return Err("cannot create PeriodicWave".to_owned());
    }
    scope
        .get_slot_mut::<PeriodicWaveStore>()
        .ok_or_else(|| "PeriodicWave state was not prepared".to_owned())?
        .records
        .insert(
            wave.get_identity_hash().get(),
            PeriodicWaveRecord {
                real,
                imaginary,
                disable_normalization,
            },
        );
    Ok(wave)
}

pub(crate) fn is_periodic_wave(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<PeriodicWaveStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn sample(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    phase: f64,
) -> Option<f32> {
    let record = scope
        .get_slot::<PeriodicWaveStore>()?
        .records
        .get(&object.get_identity_hash().get())?;
    let mut sample = 0.0_f64;
    for harmonic in 1..record.real.len() {
        let angle = phase * harmonic as f64;
        sample += f64::from(record.real[harmonic]) * angle.cos()
            + f64::from(record.imaginary[harmonic]) * angle.sin();
    }
    if !record.disable_normalization {
        let bound: f64 = record
            .real
            .iter()
            .zip(&record.imaginary)
            .skip(1)
            .map(|(real, imaginary)| f64::from(*real).hypot(f64::from(*imaginary)))
            .sum();
        if bound > 1.0 {
            sample /= bound;
        }
    }
    Some(sample.clamp(-1.0, 1.0) as f32)
}
