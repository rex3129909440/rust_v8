use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Default)]
struct RenderState {
    sample_rate: f64,
    cache: HashMap<(i32, u32), f32>,
    biquad: HashMap<(i32, u32), BiquadState>,
    iir: HashMap<(i32, u32), IirState>,
    delay: HashMap<(i32, u32), VecDeque<f32>>,
    convolver: HashMap<(i32, u32), VecDeque<f32>>,
    convolver_impulses: HashMap<(i32, u32), Arc<Vec<f32>>>,
    compressor_gain: HashMap<(i32, u32), f64>,
}

#[derive(Default)]
struct BiquadState {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

#[derive(Default)]
struct IirState {
    inputs: VecDeque<f64>,
    outputs: VecDeque<f64>,
}

pub(crate) fn render(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    buffer: v8::Local<'_, v8::Object>,
    number_of_channels: u32,
    length: u32,
    sample_rate: f64,
) {
    let Some(destination) = super::base_audio_context::destination(scope, context) else {
        return;
    };
    let mut state = RenderState {
        sample_rate,
        ..RenderState::default()
    };
    for frame in 0..length {
        state.cache.clear();
        let time = frame as f64 / sample_rate;
        for channel in 0..number_of_channels {
            let mut visiting = HashSet::new();
            let sample = render_node(scope, destination, channel, time, &mut visiting, &mut state);
            super::audio_buffer::set_sample(scope, buffer, channel, frame, sanitize_sample(sample));
        }
    }
}

fn render_node(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    channel: u32,
    time: f64,
    visiting: &mut HashSet<i32>,
    state: &mut RenderState,
) -> f32 {
    let identity = node.get_identity_hash().get();
    if let Some(sample) = state.cache.get(&(identity, channel)) {
        return *sample;
    }
    if !visiting.insert(identity) {
        return 0.0;
    }

    let source_sample = super::oscillator_node::sample_at(scope, node, time)
        .or_else(|| super::constant_source_node::sample_at(scope, node, time))
        .or_else(|| super::audio_buffer_source_node::sample_at(scope, node, channel, time));
    let result = if let Some(sample) = source_sample {
        sample
    } else if let Some(pan) = super::stereo_panner_node::pan_at(scope, node, time) {
        render_stereo_panner(scope, node, channel, time, pan, visiting, state)
    } else if let Some(spatial) = super::panner_node::spatial_parameters(scope, node, time) {
        render_spatial_panner(scope, node, channel, time, spatial, visiting, state)
    } else {
        let input = render_inputs(scope, node, channel, time, visiting, state);
        if super::analyser_node::capture_sample(scope, node, input) {
            input
        } else if let Some(gain) = super::gain_node::gain_at(scope, node, time) {
            input * gain
        } else if let Some(coefficients) =
            super::biquad_filter_node::normalized_coefficients_at(scope, node, time)
        {
            process_biquad(state, identity, channel, input, coefficients)
        } else if let Some((feedforward, feedback)) =
            super::iir_filter_node::coefficients(scope, node)
        {
            process_iir(state, identity, channel, input, &feedforward, &feedback)
        } else if let Some(delay) = super::delay_node::delay_at(scope, node, time) {
            process_delay(state, identity, channel, input, delay)
        } else if let Some(impulse) = convolver_impulse(scope, node, state, identity, channel) {
            process_convolver(state, identity, channel, input, &impulse)
        } else if let Some(parameters) =
            super::dynamics_compressor_node::parameters_at(scope, node, time)
        {
            process_compressor(scope, node, state, identity, channel, input, parameters)
        } else if let Some(shaped) = super::wave_shaper_node::shape(scope, node, input) {
            shaped
        } else {
            input
        }
    };

    visiting.remove(&identity);
    let result = sanitize_sample(result);
    state.cache.insert((identity, channel), result);
    result
}

fn convolver_impulse(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    state: &mut RenderState,
    identity: i32,
    channel: u32,
) -> Option<Arc<Vec<f32>>> {
    let key = (identity, channel);
    if let Some(impulse) = state.convolver_impulses.get(&key) {
        return Some(impulse.clone());
    }
    let impulse = Arc::new(super::convolver_node::impulse_response(
        scope, node, channel,
    )?);
    state.convolver_impulses.insert(key, impulse.clone());
    Some(impulse)
}

fn render_inputs(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    channel: u32,
    time: f64,
    visiting: &mut HashSet<i32>,
    state: &mut RenderState,
) -> f32 {
    let is_merger = super::channel_merger_node::is_channel_merger(scope, node);
    super::audio_node::incoming_connections(scope, node)
        .into_iter()
        .filter(|connection| !is_merger || connection.input == channel)
        .map(|connection| {
            let source = v8::Local::new(scope, &connection.source);
            let source_channel = if super::channel_splitter_node::is_channel_splitter(scope, source)
            {
                connection.output
            } else if is_merger {
                0
            } else {
                channel
            };
            render_node(scope, source, source_channel, time, visiting, state)
        })
        .sum()
}

fn render_stereo_panner(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    channel: u32,
    time: f64,
    pan: f32,
    visiting: &mut HashSet<i32>,
    state: &mut RenderState,
) -> f32 {
    let pan = f64::from(pan.clamp(-1.0, 1.0));
    let left = render_inputs(scope, node, 0, time, visiting, state);
    let right = render_inputs(scope, node, 1, time, visiting, state);
    if pan <= 0.0 {
        let angle = (pan + 1.0) * std::f64::consts::FRAC_PI_4;
        if channel == 0 {
            left + right * angle.cos() as f32
        } else {
            right * angle.sin() as f32
        }
    } else {
        let angle = pan * std::f64::consts::FRAC_PI_4;
        if channel == 0 {
            left * angle.cos() as f32
        } else {
            right + left * angle.sin() as f32
        }
    }
}

fn render_spatial_panner(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    channel: u32,
    time: f64,
    spatial: super::panner_node::SpatialParameters,
    visiting: &mut HashSet<i32>,
    state: &mut RenderState,
) -> f32 {
    let input = render_inputs(scope, node, 0, time, visiting, state);
    let x = f64::from(spatial.position[0]);
    let z = f64::from(spatial.position[2]);
    let pan = if x.abs() + z.abs() <= f64::EPSILON {
        0.0
    } else {
        x.atan2(-z) / std::f64::consts::FRAC_PI_2
    }
    .clamp(-1.0, 1.0);
    let angle = (pan + 1.0) * std::f64::consts::FRAC_PI_4;
    let channel_gain = if channel == 0 {
        angle.cos()
    } else {
        angle.sin()
    };
    input * spatial.distance_gain * channel_gain as f32
}

fn process_biquad(
    state: &mut RenderState,
    identity: i32,
    channel: u32,
    input: f32,
    coefficients: [f64; 5],
) -> f32 {
    let filter = state.biquad.entry((identity, channel)).or_default();
    let input = f64::from(input);
    let output =
        coefficients[0] * input + coefficients[1] * filter.x1 + coefficients[2] * filter.x2
            - coefficients[3] * filter.y1
            - coefficients[4] * filter.y2;
    filter.x2 = filter.x1;
    filter.x1 = input;
    filter.y2 = filter.y1;
    filter.y1 = output;
    output as f32
}

fn process_iir(
    state: &mut RenderState,
    identity: i32,
    channel: u32,
    input: f32,
    feedforward: &[f64],
    feedback: &[f64],
) -> f32 {
    let filter = state.iir.entry((identity, channel)).or_default();
    filter.inputs.push_front(f64::from(input));
    filter.inputs.truncate(feedforward.len());
    let mut output = feedforward
        .iter()
        .zip(&filter.inputs)
        .map(|(coefficient, sample)| coefficient * sample)
        .sum::<f64>();
    output -= feedback
        .iter()
        .skip(1)
        .zip(&filter.outputs)
        .map(|(coefficient, sample)| coefficient * sample)
        .sum::<f64>();
    output /= feedback.first().copied().unwrap_or(1.0);
    filter.outputs.push_front(output);
    filter.outputs.truncate(feedback.len().saturating_sub(1));
    output as f32
}

fn process_delay(
    state: &mut RenderState,
    identity: i32,
    channel: u32,
    input: f32,
    delay: f32,
) -> f32 {
    let delay_frames = (f64::from(delay.max(0.0)) * state.sample_rate).round() as usize;
    if delay_frames == 0 {
        state.delay.remove(&(identity, channel));
        return input;
    }
    let line = state.delay.entry((identity, channel)).or_default();
    line.push_back(input);
    if line.len() > delay_frames {
        line.pop_front().unwrap_or(0.0)
    } else {
        0.0
    }
}

fn process_convolver(
    state: &mut RenderState,
    identity: i32,
    channel: u32,
    input: f32,
    impulse: &[f32],
) -> f32 {
    if impulse.is_empty() {
        return 0.0;
    }
    let history = state.convolver.entry((identity, channel)).or_default();
    history.push_front(input);
    history.truncate(impulse.len());
    history
        .iter()
        .zip(impulse)
        .map(|(sample, coefficient)| sample * coefficient)
        .sum()
}

fn process_compressor(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    state: &mut RenderState,
    identity: i32,
    channel: u32,
    input: f32,
    parameters: super::dynamics_compressor_node::CompressorParameters,
) -> f32 {
    let input_db = 20.0 * f64::from(input.abs().max(1.0e-12)).log10();
    let threshold = f64::from(parameters.threshold);
    let knee = f64::from(parameters.knee.max(0.0));
    let ratio = f64::from(parameters.ratio.max(1.0));
    let over = input_db - threshold;
    let gain_reduction_db = if knee > 0.0 && over > -knee / 2.0 && over < knee / 2.0 {
        let position = over + knee / 2.0;
        (1.0 / ratio - 1.0) * position * position / (2.0 * knee)
    } else if over >= knee / 2.0 {
        (1.0 / ratio - 1.0) * over
    } else {
        0.0
    };
    let desired_gain = 10.0_f64.powf(gain_reduction_db / 20.0);
    let gain = state
        .compressor_gain
        .entry((identity, channel))
        .or_insert(1.0);
    let time = if desired_gain < *gain {
        f64::from(parameters.attack.max(0.0))
    } else {
        f64::from(parameters.release.max(0.0))
    };
    let coefficient = if time <= f64::EPSILON {
        0.0
    } else {
        (-1.0 / (time * state.sample_rate)).exp()
    };
    *gain = desired_gain + coefficient * (*gain - desired_gain);
    super::dynamics_compressor_node::set_reduction(scope, node, 20.0 * gain.max(1.0e-12).log10());
    (f64::from(input) * *gain) as f32
}

fn sanitize_sample(sample: f32) -> f32 {
    if sample.is_finite() {
        sample.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}
