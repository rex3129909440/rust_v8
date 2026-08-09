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
    compressor: HashMap<(i32, u32), CompressorState>,
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

struct CompressorState {
    detector_average: f32,
    compressor_gain: f32,
    metering_gain: f32,
    max_attack_compression_diff_db: f32,
    pre_delay: Vec<f32>,
    pre_delay_read_index: usize,
    pre_delay_write_index: usize,
    block_frames_remaining: u8,
    scaled_desired_gain: f32,
    envelope_rate: f32,
}

impl CompressorState {
    fn new(sample_rate: f64) -> Self {
        Self {
            detector_average: 0.0,
            compressor_gain: 1.0,
            metering_gain: 1.0,
            max_attack_compression_diff_db: -1.0,
            pre_delay: vec![0.0; 1_024],
            pre_delay_read_index: 0,
            pre_delay_write_index: (0.006 * sample_rate) as usize,
            block_frames_remaining: 0,
            scaled_desired_gain: 0.0,
            envelope_rate: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct CompressorCurve {
    linear_threshold: f32,
    slope: f32,
    knee_threshold: f32,
    knee_threshold_db: f32,
    yknee_threshold_db: f32,
    knee: f32,
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
    // Blink's DynamicsCompressorKernel is a 32-frame, look-ahead compressor.
    // Keep the kernel state between samples because the surrounding renderer
    // evaluates the graph one frame at a time.
    let sample_rate = state.sample_rate as f32;
    let curve = compressor_curve(&parameters);
    let master_linear_gain = (1.0 / compressor_saturate(1.0, curve)).powf(0.6);
    let compressor = state
        .compressor
        .entry((identity, channel))
        .or_insert_with(|| CompressorState::new(f64::from(sample_rate)));

    if compressor.block_frames_remaining == 0 {
        let desired_gain = if compressor.detector_average.is_finite() {
            compressor.detector_average
        } else {
            1.0
        };
        compressor.scaled_desired_gain =
            desired_gain.clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2;
        let mut compression_diff_db =
            linear_to_decibels(compressor.compressor_gain / compressor.scaled_desired_gain);
        if compressor.scaled_desired_gain > compressor.compressor_gain {
            compressor.max_attack_compression_diff_db = -1.0;
            if !compression_diff_db.is_finite() {
                compression_diff_db = -1.0;
            }
            let x = 0.25 * (compression_diff_db.clamp(-12.0, 0.0) + 12.0);
            let release_frames =
                adaptive_release_frames(sample_rate * parameters.release.max(0.0), x);
            compressor.envelope_rate = decibels_to_linear(5.0 / release_frames);
        } else {
            if !compression_diff_db.is_finite() {
                compression_diff_db = 1.0;
            }
            if compressor.max_attack_compression_diff_db == -1.0
                || compressor.max_attack_compression_diff_db < compression_diff_db
            {
                compressor.max_attack_compression_diff_db = compression_diff_db;
            }
            let attack_frames = parameters.attack.max(0.001) * sample_rate;
            let attenuation = compressor.max_attack_compression_diff_db.max(0.5);
            compressor.envelope_rate = 1.0 - (0.25 / attenuation).powf(1.0 / attack_frames);
        }
        compressor.block_frames_remaining = 32;
    }

    let delayed = compressor.pre_delay[compressor.pre_delay_read_index];
    compressor.pre_delay[compressor.pre_delay_write_index] = input;
    compressor.pre_delay_read_index = (compressor.pre_delay_read_index + 1) & 1_023;
    compressor.pre_delay_write_index = (compressor.pre_delay_write_index + 1) & 1_023;

    let absolute_input = input.abs();
    let shaped_input = compressor_saturate(absolute_input, curve);
    let attenuation = if absolute_input <= 0.0001 {
        1.0
    } else {
        shaped_input / absolute_input
    };
    let attenuation_db = (-linear_to_decibels(attenuation)).max(2.0);
    let detector_release_frames = 0.0025 * sample_rate;
    let detector_release_rate = decibels_to_linear(attenuation_db / detector_release_frames) - 1.0;
    let detector_rate = if attenuation > compressor.detector_average {
        detector_release_rate
    } else {
        1.0
    };
    compressor.detector_average += (attenuation - compressor.detector_average) * detector_rate;
    compressor.detector_average = compressor.detector_average.min(1.0);
    if !compressor.detector_average.is_finite() {
        compressor.detector_average = 1.0;
    }

    if compressor.envelope_rate < 1.0 {
        compressor.compressor_gain += (compressor.scaled_desired_gain - compressor.compressor_gain)
            * compressor.envelope_rate;
    } else {
        compressor.compressor_gain =
            (compressor.compressor_gain * compressor.envelope_rate).min(1.0);
    }
    let post_warp_gain = (std::f32::consts::FRAC_PI_2 * compressor.compressor_gain).sin();
    let real_gain_db = linear_to_decibels(post_warp_gain);
    let metering_release = 1.0 - (-1.0 / (sample_rate * 0.325)).exp();
    if real_gain_db < compressor.metering_gain {
        compressor.metering_gain = real_gain_db;
    } else {
        compressor.metering_gain += (real_gain_db - compressor.metering_gain) * metering_release;
    }
    compressor.block_frames_remaining -= 1;
    super::dynamics_compressor_node::set_reduction(
        scope,
        node,
        f64::from(compressor.metering_gain),
    );
    delayed * master_linear_gain * post_warp_gain
}

fn compressor_curve(
    parameters: &super::dynamics_compressor_node::CompressorParameters,
) -> CompressorCurve {
    let threshold_db = parameters.threshold;
    let knee_db = parameters.knee.max(0.0);
    let slope = 1.0 / parameters.ratio.max(1.0);
    let linear_threshold = decibels_to_linear(threshold_db);
    let knee_threshold_db = threshold_db + knee_db;
    let knee_threshold = decibels_to_linear(knee_threshold_db);
    let knee_curve = |x: f32, knee: f32| {
        if x < linear_threshold {
            x
        } else {
            linear_threshold + (1.0 - (-knee * (x - linear_threshold)).exp()) / knee
        }
    };
    let slope_at = |x: f32, knee: f32| {
        let x2 = x * 1.001;
        (linear_to_decibels(knee_curve(x2, knee)) - linear_to_decibels(knee_curve(x, knee)))
            / (linear_to_decibels(x2) - linear_to_decibels(x))
    };
    let mut minimum_k = 0.1_f32;
    let mut maximum_k = 10_000.0_f32;
    let mut knee = 5.0_f32;
    for _ in 0..15 {
        if slope_at(knee_threshold, knee) < slope {
            maximum_k = knee;
        } else {
            minimum_k = knee;
        }
        knee = (minimum_k * maximum_k).sqrt();
    }
    let yknee_threshold_db = linear_to_decibels(knee_curve(knee_threshold, knee));
    CompressorCurve {
        linear_threshold,
        slope,
        knee_threshold,
        knee_threshold_db,
        yknee_threshold_db,
        knee,
    }
}

fn compressor_saturate(input: f32, curve: CompressorCurve) -> f32 {
    if input < curve.knee_threshold {
        if input < curve.linear_threshold {
            input
        } else {
            curve.linear_threshold
                + (1.0 - (-curve.knee * (input - curve.linear_threshold)).exp()) / curve.knee
        }
    } else {
        decibels_to_linear(
            curve.yknee_threshold_db
                + curve.slope * (linear_to_decibels(input) - curve.knee_threshold_db),
        )
    }
}

fn adaptive_release_frames(release_frames: f32, x: f32) -> f32 {
    let y1 = release_frames * 0.09;
    let y2 = release_frames * 0.16;
    let y3 = release_frames * 0.42;
    let y4 = release_frames * 0.98;
    let a = 0.999_999_999_999_999_8 * y1 + 1.843_221_968_432_392_3e-16 * y2
        - 1.937_339_435_167_642_3e-16 * y3
        + 8.824_516_011_816_245e-18 * y4;
    let b = -1.578_832_035_284_588_8 * y1 + 2.330_583_703_207_428_6 * y2
        - 0.914_119_420_484_042_9 * y3
        + 0.162_367_752_561_203_2 * y4;
    let c = 0.533_414_286_910_642_4 * y1 - 1.272_736_789_213_631 * y2
        + 0.925_885_604_220_751_2 * y3
        - 0.186_563_101_917_762_26 * y4;
    let d = 0.087_834_631_382_072_34 * y1 - 0.169_416_296_792_562_2 * y2
        + 0.085_880_579_515_952_72 * y3
        - 0.004_298_914_105_462_83 * y4;
    let e = -0.042_416_883_008_123_074 * y1 + 0.111_569_382_798_760_2 * y2
        - 0.097_646_763_252_658_72 * y3
        + 0.028_494_263_462_021_576 * y4;
    let x2 = x * x;
    a + b * x + c * x2 + d * x2 * x + e * x2 * x2
}

fn decibels_to_linear(decibels: f32) -> f32 {
    10.0_f32.powf(0.05 * decibels)
}

fn linear_to_decibels(linear: f32) -> f32 {
    20.0 * linear.log10()
}

fn sanitize_sample(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}
