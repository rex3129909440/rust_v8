use std::collections::{HashMap, HashSet};

const NO_ERROR: u32 = 0;
const INVALID_ENUM: u32 = 0x0500;
const INVALID_VALUE: u32 = 0x0501;
const INVALID_OPERATION: u32 = 0x0502;

#[derive(Default)]
pub(crate) struct WebGl2RenderingContextStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ContextRecord>,
}

#[derive(Clone)]
struct QueryRecord {
    object: v8::Global<v8::Object>,
    target: u32,
    active: bool,
    available: bool,
    result: u32,
}

#[derive(Clone, Default)]
struct SamplerRecord {
    parameters: HashMap<u32, f64>,
}

#[derive(Clone, Default)]
struct TransformFeedbackRecord {
    active: bool,
    paused: bool,
}

#[derive(Clone)]
struct SyncRecord {
    object: v8::Global<v8::Object>,
    condition: u32,
    flags: u32,
    signaled: bool,
}

#[derive(Clone)]
struct ContextRecord {
    canvas: Option<v8::Global<v8::Object>>,
    width: u32,
    height: u32,
    drawing_buffer_color_space: String,
    unpack_color_space: String,
    error: u32,
    buffers: HashSet<i32>,
    framebuffers: HashSet<i32>,
    programs: HashSet<i32>,
    renderbuffers: HashSet<i32>,
    shaders: HashSet<i32>,
    textures: HashSet<i32>,
    queries: HashMap<i32, QueryRecord>,
    samplers: HashMap<i32, SamplerRecord>,
    syncs: HashMap<i32, SyncRecord>,
    transform_feedbacks: HashMap<i32, TransformFeedbackRecord>,
    vertex_arrays: HashSet<i32>,
    active_queries: HashMap<u32, i32>,
    bound_samplers: HashMap<u32, i32>,
    bound_transform_feedback: Option<i32>,
    bound_vertex_array: Option<i32>,
    read_buffer: u32,
    draw_buffers: Vec<u32>,
    operation_count: u64,
    extensions: HashMap<String, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WebGl2RenderingContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebGL2RenderingContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<WebGl2RenderingContextStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WebGL2RenderingContext",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canvas", get_canvas)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "drawingBufferWidth",
        get_drawing_buffer_width,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "drawingBufferHeight",
        get_drawing_buffer_height,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "drawingBufferColorSpace",
        get_drawing_buffer_color_space,
        set_drawing_buffer_color_space,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "unpackColorSpace",
        get_unpack_color_space,
        set_unpack_color_space,
    )?;
    define_all_constants(scope, prototype)?;
    define_all_methods(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, prototype, "makeXRCompatible", 0, make_xr_compatible)?;
    define_all_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WebGl2RenderingContextStore>()
        .ok_or_else(|| "WebGL2RenderingContext state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn make_xr_compatible(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let undefined = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
        result.set(promise.into());
    }
}

fn define_all_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    super::webgl_rendering_context::define_core_constants(scope, object)?;
    define_constants_301_through_393(scope, object)?;
    define_constants_394_through_474(scope, object)?;
    define_constants_475_through_563(scope, object)
}

fn constant(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: u32,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let value = v8::Integer::new_from_unsigned(scope, value);
    match object.define_own_property(
        scope,
        key.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define WebGL2 constant {name}")),
    }
}

fn signed_constant(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, name, value)
}

fn define_constants_301_through_393(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    constant(scope, object, "READ_BUFFER", 3074)?;
    constant(scope, object, "UNPACK_ROW_LENGTH", 3314)?;
    constant(scope, object, "UNPACK_SKIP_ROWS", 3315)?;
    constant(scope, object, "UNPACK_SKIP_PIXELS", 3316)?;
    constant(scope, object, "PACK_ROW_LENGTH", 3330)?;
    constant(scope, object, "PACK_SKIP_ROWS", 3331)?;
    constant(scope, object, "PACK_SKIP_PIXELS", 3332)?;
    constant(scope, object, "COLOR", 6144)?;
    constant(scope, object, "DEPTH", 6145)?;
    constant(scope, object, "STENCIL", 6146)?;
    constant(scope, object, "RED", 6403)?;
    constant(scope, object, "RGB8", 32849)?;
    constant(scope, object, "RGBA8", 32856)?;
    constant(scope, object, "RGB10_A2", 32857)?;
    constant(scope, object, "TEXTURE_BINDING_3D", 32874)?;
    constant(scope, object, "UNPACK_SKIP_IMAGES", 32877)?;
    constant(scope, object, "UNPACK_IMAGE_HEIGHT", 32878)?;
    constant(scope, object, "TEXTURE_3D", 32879)?;
    constant(scope, object, "TEXTURE_WRAP_R", 32882)?;
    constant(scope, object, "MAX_3D_TEXTURE_SIZE", 32883)?;
    constant(scope, object, "UNSIGNED_INT_2_10_10_10_REV", 33640)?;
    constant(scope, object, "MAX_ELEMENTS_VERTICES", 33000)?;
    constant(scope, object, "MAX_ELEMENTS_INDICES", 33001)?;
    constant(scope, object, "TEXTURE_MIN_LOD", 33082)?;
    constant(scope, object, "TEXTURE_MAX_LOD", 33083)?;
    constant(scope, object, "TEXTURE_BASE_LEVEL", 33084)?;
    constant(scope, object, "TEXTURE_MAX_LEVEL", 33085)?;
    constant(scope, object, "MIN", 32775)?;
    constant(scope, object, "MAX", 32776)?;
    constant(scope, object, "DEPTH_COMPONENT24", 33190)?;
    constant(scope, object, "MAX_TEXTURE_LOD_BIAS", 34045)?;
    constant(scope, object, "TEXTURE_COMPARE_MODE", 34892)?;
    constant(scope, object, "TEXTURE_COMPARE_FUNC", 34893)?;
    constant(scope, object, "CURRENT_QUERY", 34917)?;
    constant(scope, object, "QUERY_RESULT", 34918)?;
    constant(scope, object, "QUERY_RESULT_AVAILABLE", 34919)?;
    constant(scope, object, "STREAM_READ", 35041)?;
    constant(scope, object, "STREAM_COPY", 35042)?;
    constant(scope, object, "STATIC_READ", 35045)?;
    constant(scope, object, "STATIC_COPY", 35046)?;
    constant(scope, object, "DYNAMIC_READ", 35049)?;
    constant(scope, object, "DYNAMIC_COPY", 35050)?;
    constant(scope, object, "MAX_DRAW_BUFFERS", 34852)?;
    constant(scope, object, "DRAW_BUFFER0", 34853)?;
    constant(scope, object, "DRAW_BUFFER1", 34854)?;
    constant(scope, object, "DRAW_BUFFER2", 34855)?;
    constant(scope, object, "DRAW_BUFFER3", 34856)?;
    constant(scope, object, "DRAW_BUFFER4", 34857)?;
    constant(scope, object, "DRAW_BUFFER5", 34858)?;
    constant(scope, object, "DRAW_BUFFER6", 34859)?;
    constant(scope, object, "DRAW_BUFFER7", 34860)?;
    constant(scope, object, "DRAW_BUFFER8", 34861)?;
    constant(scope, object, "DRAW_BUFFER9", 34862)?;
    constant(scope, object, "DRAW_BUFFER10", 34863)?;
    constant(scope, object, "DRAW_BUFFER11", 34864)?;
    constant(scope, object, "DRAW_BUFFER12", 34865)?;
    constant(scope, object, "DRAW_BUFFER13", 34866)?;
    constant(scope, object, "DRAW_BUFFER14", 34867)?;
    constant(scope, object, "DRAW_BUFFER15", 34868)?;
    constant(scope, object, "MAX_FRAGMENT_UNIFORM_COMPONENTS", 35657)?;
    constant(scope, object, "MAX_VERTEX_UNIFORM_COMPONENTS", 35658)?;
    constant(scope, object, "SAMPLER_3D", 35679)?;
    constant(scope, object, "SAMPLER_2D_SHADOW", 35682)?;
    constant(scope, object, "FRAGMENT_SHADER_DERIVATIVE_HINT", 35723)?;
    constant(scope, object, "PIXEL_PACK_BUFFER", 35051)?;
    constant(scope, object, "PIXEL_UNPACK_BUFFER", 35052)?;
    constant(scope, object, "PIXEL_PACK_BUFFER_BINDING", 35053)?;
    constant(scope, object, "PIXEL_UNPACK_BUFFER_BINDING", 35055)?;
    constant(scope, object, "FLOAT_MAT2x3", 35685)?;
    constant(scope, object, "FLOAT_MAT2x4", 35686)?;
    constant(scope, object, "FLOAT_MAT3x2", 35687)?;
    constant(scope, object, "FLOAT_MAT3x4", 35688)?;
    constant(scope, object, "FLOAT_MAT4x2", 35689)?;
    constant(scope, object, "FLOAT_MAT4x3", 35690)?;
    constant(scope, object, "SRGB", 35904)?;
    constant(scope, object, "SRGB8", 35905)?;
    constant(scope, object, "SRGB8_ALPHA8", 35907)?;
    constant(scope, object, "COMPARE_REF_TO_TEXTURE", 34894)?;
    constant(scope, object, "RGBA32F", 34836)?;
    constant(scope, object, "RGB32F", 34837)?;
    constant(scope, object, "RGBA16F", 34842)?;
    constant(scope, object, "RGB16F", 34843)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_INTEGER", 35069)?;
    constant(scope, object, "MAX_ARRAY_TEXTURE_LAYERS", 35071)?;
    constant(scope, object, "MIN_PROGRAM_TEXEL_OFFSET", 35076)?;
    constant(scope, object, "MAX_PROGRAM_TEXEL_OFFSET", 35077)?;
    constant(scope, object, "MAX_VARYING_COMPONENTS", 35659)?;
    constant(scope, object, "TEXTURE_2D_ARRAY", 35866)?;
    constant(scope, object, "TEXTURE_BINDING_2D_ARRAY", 35869)?;
    constant(scope, object, "R11F_G11F_B10F", 35898)?;
    constant(scope, object, "UNSIGNED_INT_10F_11F_11F_REV", 35899)?;
    constant(scope, object, "RGB9_E5", 35901)?;
    constant(scope, object, "UNSIGNED_INT_5_9_9_9_REV", 35902)
}

fn define_constants_394_through_474(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    constant(scope, object, "TRANSFORM_FEEDBACK_BUFFER_MODE", 35967)?;
    constant(
        scope,
        object,
        "MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS",
        35968,
    )?;
    constant(scope, object, "TRANSFORM_FEEDBACK_VARYINGS", 35971)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_BUFFER_START", 35972)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_BUFFER_SIZE", 35973)?;
    constant(
        scope,
        object,
        "TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN",
        35976,
    )?;
    constant(scope, object, "RASTERIZER_DISCARD", 35977)?;
    constant(
        scope,
        object,
        "MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS",
        35978,
    )?;
    constant(
        scope,
        object,
        "MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS",
        35979,
    )?;
    constant(scope, object, "INTERLEAVED_ATTRIBS", 35980)?;
    constant(scope, object, "SEPARATE_ATTRIBS", 35981)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_BUFFER", 35982)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_BUFFER_BINDING", 35983)?;
    constant(scope, object, "RGBA32UI", 36208)?;
    constant(scope, object, "RGB32UI", 36209)?;
    constant(scope, object, "RGBA16UI", 36214)?;
    constant(scope, object, "RGB16UI", 36215)?;
    constant(scope, object, "RGBA8UI", 36220)?;
    constant(scope, object, "RGB8UI", 36221)?;
    constant(scope, object, "RGBA32I", 36226)?;
    constant(scope, object, "RGB32I", 36227)?;
    constant(scope, object, "RGBA16I", 36232)?;
    constant(scope, object, "RGB16I", 36233)?;
    constant(scope, object, "RGBA8I", 36238)?;
    constant(scope, object, "RGB8I", 36239)?;
    constant(scope, object, "RED_INTEGER", 36244)?;
    constant(scope, object, "RGB_INTEGER", 36248)?;
    constant(scope, object, "RGBA_INTEGER", 36249)?;
    constant(scope, object, "SAMPLER_2D_ARRAY", 36289)?;
    constant(scope, object, "SAMPLER_2D_ARRAY_SHADOW", 36292)?;
    constant(scope, object, "SAMPLER_CUBE_SHADOW", 36293)?;
    constant(scope, object, "UNSIGNED_INT_VEC2", 36294)?;
    constant(scope, object, "UNSIGNED_INT_VEC3", 36295)?;
    constant(scope, object, "UNSIGNED_INT_VEC4", 36296)?;
    constant(scope, object, "INT_SAMPLER_2D", 36298)?;
    constant(scope, object, "INT_SAMPLER_3D", 36299)?;
    constant(scope, object, "INT_SAMPLER_CUBE", 36300)?;
    constant(scope, object, "INT_SAMPLER_2D_ARRAY", 36303)?;
    constant(scope, object, "UNSIGNED_INT_SAMPLER_2D", 36306)?;
    constant(scope, object, "UNSIGNED_INT_SAMPLER_3D", 36307)?;
    constant(scope, object, "UNSIGNED_INT_SAMPLER_CUBE", 36308)?;
    constant(scope, object, "UNSIGNED_INT_SAMPLER_2D_ARRAY", 36311)?;
    constant(scope, object, "DEPTH_COMPONENT32F", 36012)?;
    constant(scope, object, "DEPTH32F_STENCIL8", 36013)?;
    constant(scope, object, "FLOAT_32_UNSIGNED_INT_24_8_REV", 36269)?;
    constant(
        scope,
        object,
        "FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING",
        33296,
    )?;
    constant(
        scope,
        object,
        "FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE",
        33297,
    )?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_RED_SIZE", 33298)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_GREEN_SIZE", 33299)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_BLUE_SIZE", 33300)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE", 33301)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE", 33302)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE", 33303)?;
    constant(scope, object, "FRAMEBUFFER_DEFAULT", 33304)?;
    constant(scope, object, "UNSIGNED_INT_24_8", 34042)?;
    constant(scope, object, "DEPTH24_STENCIL8", 35056)?;
    constant(scope, object, "UNSIGNED_NORMALIZED", 35863)?;
    constant(scope, object, "DRAW_FRAMEBUFFER_BINDING", 36006)?;
    constant(scope, object, "READ_FRAMEBUFFER", 36008)?;
    constant(scope, object, "DRAW_FRAMEBUFFER", 36009)?;
    constant(scope, object, "READ_FRAMEBUFFER_BINDING", 36010)?;
    constant(scope, object, "RENDERBUFFER_SAMPLES", 36011)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER", 36052)?;
    constant(scope, object, "MAX_COLOR_ATTACHMENTS", 36063)?;
    constant(scope, object, "COLOR_ATTACHMENT1", 36065)?;
    constant(scope, object, "COLOR_ATTACHMENT2", 36066)?;
    constant(scope, object, "COLOR_ATTACHMENT3", 36067)?;
    constant(scope, object, "COLOR_ATTACHMENT4", 36068)?;
    constant(scope, object, "COLOR_ATTACHMENT5", 36069)?;
    constant(scope, object, "COLOR_ATTACHMENT6", 36070)?;
    constant(scope, object, "COLOR_ATTACHMENT7", 36071)?;
    constant(scope, object, "COLOR_ATTACHMENT8", 36072)?;
    constant(scope, object, "COLOR_ATTACHMENT9", 36073)?;
    constant(scope, object, "COLOR_ATTACHMENT10", 36074)?;
    constant(scope, object, "COLOR_ATTACHMENT11", 36075)?;
    constant(scope, object, "COLOR_ATTACHMENT12", 36076)?;
    constant(scope, object, "COLOR_ATTACHMENT13", 36077)?;
    constant(scope, object, "COLOR_ATTACHMENT14", 36078)?;
    constant(scope, object, "COLOR_ATTACHMENT15", 36079)?;
    constant(scope, object, "FRAMEBUFFER_INCOMPLETE_MULTISAMPLE", 36182)?;
    constant(scope, object, "MAX_SAMPLES", 36183)
}

fn define_constants_475_through_563(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    constant(scope, object, "HALF_FLOAT", 5131)?;
    constant(scope, object, "RG", 33319)?;
    constant(scope, object, "RG_INTEGER", 33320)?;
    constant(scope, object, "R8", 33321)?;
    constant(scope, object, "RG8", 33323)?;
    constant(scope, object, "R16F", 33325)?;
    constant(scope, object, "R32F", 33326)?;
    constant(scope, object, "RG16F", 33327)?;
    constant(scope, object, "RG32F", 33328)?;
    constant(scope, object, "R8I", 33329)?;
    constant(scope, object, "R8UI", 33330)?;
    constant(scope, object, "R16I", 33331)?;
    constant(scope, object, "R16UI", 33332)?;
    constant(scope, object, "R32I", 33333)?;
    constant(scope, object, "R32UI", 33334)?;
    constant(scope, object, "RG8I", 33335)?;
    constant(scope, object, "RG8UI", 33336)?;
    constant(scope, object, "RG16I", 33337)?;
    constant(scope, object, "RG16UI", 33338)?;
    constant(scope, object, "RG32I", 33339)?;
    constant(scope, object, "RG32UI", 33340)?;
    constant(scope, object, "VERTEX_ARRAY_BINDING", 34229)?;
    constant(scope, object, "R8_SNORM", 36756)?;
    constant(scope, object, "RG8_SNORM", 36757)?;
    constant(scope, object, "RGB8_SNORM", 36758)?;
    constant(scope, object, "RGBA8_SNORM", 36759)?;
    constant(scope, object, "SIGNED_NORMALIZED", 36764)?;
    constant(scope, object, "COPY_READ_BUFFER", 36662)?;
    constant(scope, object, "COPY_WRITE_BUFFER", 36663)?;
    constant(scope, object, "COPY_READ_BUFFER_BINDING", 36662)?;
    constant(scope, object, "COPY_WRITE_BUFFER_BINDING", 36663)?;
    constant(scope, object, "UNIFORM_BUFFER", 35345)?;
    constant(scope, object, "UNIFORM_BUFFER_BINDING", 35368)?;
    constant(scope, object, "UNIFORM_BUFFER_START", 35369)?;
    constant(scope, object, "UNIFORM_BUFFER_SIZE", 35370)?;
    constant(scope, object, "MAX_VERTEX_UNIFORM_BLOCKS", 35371)?;
    constant(scope, object, "MAX_FRAGMENT_UNIFORM_BLOCKS", 35373)?;
    constant(scope, object, "MAX_COMBINED_UNIFORM_BLOCKS", 35374)?;
    constant(scope, object, "MAX_UNIFORM_BUFFER_BINDINGS", 35375)?;
    constant(scope, object, "MAX_UNIFORM_BLOCK_SIZE", 35376)?;
    constant(
        scope,
        object,
        "MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS",
        35377,
    )?;
    constant(
        scope,
        object,
        "MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS",
        35379,
    )?;
    constant(scope, object, "UNIFORM_BUFFER_OFFSET_ALIGNMENT", 35380)?;
    constant(scope, object, "ACTIVE_UNIFORM_BLOCKS", 35382)?;
    constant(scope, object, "UNIFORM_TYPE", 35383)?;
    constant(scope, object, "UNIFORM_SIZE", 35384)?;
    constant(scope, object, "UNIFORM_BLOCK_INDEX", 35386)?;
    constant(scope, object, "UNIFORM_OFFSET", 35387)?;
    constant(scope, object, "UNIFORM_ARRAY_STRIDE", 35388)?;
    constant(scope, object, "UNIFORM_MATRIX_STRIDE", 35389)?;
    constant(scope, object, "UNIFORM_IS_ROW_MAJOR", 35390)?;
    constant(scope, object, "UNIFORM_BLOCK_BINDING", 35391)?;
    constant(scope, object, "UNIFORM_BLOCK_DATA_SIZE", 35392)?;
    constant(scope, object, "UNIFORM_BLOCK_ACTIVE_UNIFORMS", 35394)?;
    constant(scope, object, "UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES", 35395)?;
    constant(
        scope,
        object,
        "UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER",
        35396,
    )?;
    constant(
        scope,
        object,
        "UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER",
        35398,
    )?;
    constant(scope, object, "INVALID_INDEX", u32::MAX)?;
    constant(scope, object, "MAX_VERTEX_OUTPUT_COMPONENTS", 37154)?;
    constant(scope, object, "MAX_FRAGMENT_INPUT_COMPONENTS", 37157)?;
    constant(scope, object, "MAX_SERVER_WAIT_TIMEOUT", 37137)?;
    constant(scope, object, "OBJECT_TYPE", 37138)?;
    constant(scope, object, "SYNC_CONDITION", 37139)?;
    constant(scope, object, "SYNC_STATUS", 37140)?;
    constant(scope, object, "SYNC_FLAGS", 37141)?;
    constant(scope, object, "SYNC_FENCE", 37142)?;
    constant(scope, object, "SYNC_GPU_COMMANDS_COMPLETE", 37143)?;
    constant(scope, object, "UNSIGNALED", 37144)?;
    constant(scope, object, "SIGNALED", 37145)?;
    constant(scope, object, "ALREADY_SIGNALED", 37146)?;
    constant(scope, object, "TIMEOUT_EXPIRED", 37147)?;
    constant(scope, object, "CONDITION_SATISFIED", 37148)?;
    constant(scope, object, "WAIT_FAILED", 37149)?;
    constant(scope, object, "SYNC_FLUSH_COMMANDS_BIT", 1)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_DIVISOR", 35070)?;
    constant(scope, object, "ANY_SAMPLES_PASSED", 35887)?;
    constant(scope, object, "ANY_SAMPLES_PASSED_CONSERVATIVE", 36202)?;
    constant(scope, object, "SAMPLER_BINDING", 35097)?;
    constant(scope, object, "RGB10_A2UI", 36975)?;
    constant(scope, object, "INT_2_10_10_10_REV", 36255)?;
    constant(scope, object, "TRANSFORM_FEEDBACK", 36386)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_PAUSED", 36387)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_ACTIVE", 36388)?;
    constant(scope, object, "TRANSFORM_FEEDBACK_BINDING", 36389)?;
    constant(scope, object, "TEXTURE_IMMUTABLE_FORMAT", 37167)?;
    constant(scope, object, "MAX_ELEMENT_INDEX", 36203)?;
    constant(scope, object, "TEXTURE_IMMUTABLE_LEVELS", 33503)?;
    signed_constant(scope, object, "TIMEOUT_IGNORED", -1)?;
    constant(scope, object, "MAX_CLIENT_WAIT_TIMEOUT_WEBGL", 37447)
}

fn method(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
    length: i32,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, name, length, callback)
}

fn define_all_methods(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    method(scope, prototype, "activeTexture", 1, operation)?;
    method(scope, prototype, "attachShader", 2, operation)?;
    method(scope, prototype, "beginQuery", 2, begin_query)?;
    method(
        scope,
        prototype,
        "beginTransformFeedback",
        1,
        begin_transform_feedback,
    )?;
    method(scope, prototype, "bindAttribLocation", 3, operation)?;
    method(scope, prototype, "bindBufferBase", 3, operation)?;
    method(scope, prototype, "bindBufferRange", 5, operation)?;
    method(scope, prototype, "bindRenderbuffer", 2, operation)?;
    method(scope, prototype, "bindSampler", 2, bind_sampler)?;
    method(
        scope,
        prototype,
        "bindTransformFeedback",
        2,
        bind_transform_feedback,
    )?;
    method(scope, prototype, "bindVertexArray", 1, bind_vertex_array)?;
    method(scope, prototype, "blendColor", 4, operation)?;
    method(scope, prototype, "blendEquation", 1, operation)?;
    method(scope, prototype, "blendEquationSeparate", 2, operation)?;
    method(scope, prototype, "blendFunc", 2, operation)?;
    method(scope, prototype, "blendFuncSeparate", 4, operation)?;
    method(scope, prototype, "blitFramebuffer", 10, operation)?;
    method(scope, prototype, "bufferData", 3, operation)?;
    method(scope, prototype, "bufferSubData", 3, operation)?;
    method(
        scope,
        prototype,
        "checkFramebufferStatus",
        1,
        check_framebuffer_status,
    )?;
    method(scope, prototype, "clientWaitSync", 3, client_wait_sync)?;
    method(scope, prototype, "compileShader", 1, operation)?;
    method(scope, prototype, "compressedTexImage2D", 7, operation)?;
    method(scope, prototype, "compressedTexImage3D", 8, operation)?;
    method(scope, prototype, "compressedTexSubImage2D", 8, operation)?;
    method(scope, prototype, "compressedTexSubImage3D", 10, operation)?;
    method(scope, prototype, "copyBufferSubData", 5, operation)?;
    method(scope, prototype, "copyTexImage2D", 8, operation)?;
    method(scope, prototype, "copyTexSubImage2D", 8, operation)?;
    method(scope, prototype, "copyTexSubImage3D", 9, operation)?;
    method(scope, prototype, "createBuffer", 0, create_buffer)?;
    method(scope, prototype, "createFramebuffer", 0, create_framebuffer)?;
    method(scope, prototype, "createProgram", 0, create_program)?;
    method(scope, prototype, "createQuery", 0, create_query)?;
    method(
        scope,
        prototype,
        "createRenderbuffer",
        0,
        create_renderbuffer,
    )?;
    method(scope, prototype, "createSampler", 0, create_sampler)?;
    method(scope, prototype, "createShader", 1, create_shader)?;
    method(scope, prototype, "createTexture", 0, create_texture)?;
    method(
        scope,
        prototype,
        "createTransformFeedback",
        0,
        create_transform_feedback,
    )?;
    method(
        scope,
        prototype,
        "createVertexArray",
        0,
        create_vertex_array,
    )?;
    method(scope, prototype, "cullFace", 1, operation)?;
    method(scope, prototype, "deleteBuffer", 1, delete_buffer)?;
    method(scope, prototype, "deleteFramebuffer", 1, delete_framebuffer)?;
    method(scope, prototype, "deleteProgram", 1, delete_program)?;
    method(scope, prototype, "deleteQuery", 1, delete_query)?;
    method(
        scope,
        prototype,
        "deleteRenderbuffer",
        1,
        delete_renderbuffer,
    )?;
    method(scope, prototype, "deleteSampler", 1, delete_sampler)?;
    method(scope, prototype, "deleteShader", 1, delete_shader)?;
    method(scope, prototype, "deleteSync", 1, delete_sync)?;
    method(scope, prototype, "deleteTexture", 1, delete_texture)?;
    method(
        scope,
        prototype,
        "deleteTransformFeedback",
        1,
        delete_transform_feedback,
    )?;
    method(
        scope,
        prototype,
        "deleteVertexArray",
        1,
        delete_vertex_array,
    )?;
    method(scope, prototype, "depthFunc", 1, operation)?;
    method(scope, prototype, "depthMask", 1, operation)?;
    method(scope, prototype, "depthRange", 2, operation)?;
    method(scope, prototype, "detachShader", 2, operation)?;
    method(scope, prototype, "disable", 1, operation)?;
    method(scope, prototype, "drawArraysInstanced", 4, operation)?;
    method(scope, prototype, "drawElementsInstanced", 5, operation)?;
    method(scope, prototype, "drawRangeElements", 6, operation)?;
    method(scope, prototype, "enable", 1, operation)?;
    method(scope, prototype, "endQuery", 1, end_query)?;
    method(
        scope,
        prototype,
        "endTransformFeedback",
        0,
        end_transform_feedback,
    )?;
    method(scope, prototype, "fenceSync", 2, fence_sync)?;
    method(scope, prototype, "finish", 0, operation)?;
    method(scope, prototype, "flush", 0, operation)?;
    method(scope, prototype, "framebufferRenderbuffer", 4, operation)?;
    method(scope, prototype, "framebufferTexture2D", 5, operation)?;
    method(scope, prototype, "framebufferTextureLayer", 5, operation)?;
    method(scope, prototype, "frontFace", 1, operation)?;
    method(scope, prototype, "generateMipmap", 1, operation)?;
    method(scope, prototype, "getActiveAttrib", 2, return_null)?;
    method(scope, prototype, "getActiveUniform", 2, return_null)?;
    method(
        scope,
        prototype,
        "getActiveUniformBlockName",
        2,
        return_null,
    )?;
    method(
        scope,
        prototype,
        "getActiveUniformBlockParameter",
        3,
        return_null,
    )?;
    method(scope, prototype, "getActiveUniforms", 3, return_empty_array)?;
    method(
        scope,
        prototype,
        "getAttachedShaders",
        1,
        return_empty_array,
    )?;
    method(
        scope,
        prototype,
        "getAttribLocation",
        2,
        return_negative_one,
    )?;
    method(scope, prototype, "getBufferParameter", 2, return_null)?;
    method(scope, prototype, "getBufferSubData", 3, operation)?;
    method(
        scope,
        prototype,
        "getContextAttributes",
        0,
        get_context_attributes,
    )?;
    method(scope, prototype, "getError", 0, get_error)?;
    method(scope, prototype, "getExtension", 1, get_extension)?;
    method(
        scope,
        prototype,
        "getFragDataLocation",
        2,
        return_negative_one,
    )?;
    method(
        scope,
        prototype,
        "getFramebufferAttachmentParameter",
        3,
        return_null,
    )?;
    method(scope, prototype, "getIndexedParameter", 2, return_null)?;
    method(
        scope,
        prototype,
        "getInternalformatParameter",
        3,
        get_internalformat_parameter,
    )?;
    method(scope, prototype, "getParameter", 1, get_parameter)?;
    method(
        scope,
        prototype,
        "getProgramInfoLog",
        1,
        return_empty_string,
    )?;
    method(scope, prototype, "getProgramParameter", 2, return_null)?;
    method(scope, prototype, "getQuery", 2, get_query)?;
    method(
        scope,
        prototype,
        "getQueryParameter",
        2,
        get_query_parameter,
    )?;
    method(scope, prototype, "getRenderbufferParameter", 2, return_null)?;
    method(
        scope,
        prototype,
        "getSamplerParameter",
        2,
        get_sampler_parameter,
    )?;
    method(scope, prototype, "getShaderInfoLog", 1, return_empty_string)?;
    method(scope, prototype, "getShaderParameter", 2, return_null)?;
    method(
        scope,
        prototype,
        "getShaderPrecisionFormat",
        2,
        get_shader_precision_format,
    )?;
    method(scope, prototype, "getShaderSource", 1, return_null)?;
    method(
        scope,
        prototype,
        "getSupportedExtensions",
        0,
        get_supported_extensions,
    )?;
    method(scope, prototype, "getSyncParameter", 2, get_sync_parameter)?;
    method(scope, prototype, "getTexParameter", 2, return_null)?;
    method(
        scope,
        prototype,
        "getTransformFeedbackVarying",
        2,
        return_null,
    )?;
    method(scope, prototype, "getUniform", 2, return_null)?;
    method(
        scope,
        prototype,
        "getUniformBlockIndex",
        2,
        return_invalid_index,
    )?;
    method(scope, prototype, "getUniformIndices", 2, return_empty_array)?;
    method(scope, prototype, "getUniformLocation", 2, return_null)?;
    method(scope, prototype, "getVertexAttrib", 2, return_null)?;
    method(scope, prototype, "getVertexAttribOffset", 2, return_zero)?;
    method(scope, prototype, "hint", 2, operation)?;
    method(scope, prototype, "invalidateFramebuffer", 2, operation)?;
    method(scope, prototype, "invalidateSubFramebuffer", 6, operation)?;
    method(scope, prototype, "isBuffer", 1, is_buffer)?;
    method(scope, prototype, "isContextLost", 0, return_false)?;
    method(scope, prototype, "isEnabled", 1, return_false)?;
    method(scope, prototype, "isFramebuffer", 1, is_framebuffer)?;
    method(scope, prototype, "isProgram", 1, is_program)?;
    method(scope, prototype, "isQuery", 1, is_query)?;
    method(scope, prototype, "isRenderbuffer", 1, is_renderbuffer)?;
    method(scope, prototype, "isSampler", 1, is_sampler)?;
    method(scope, prototype, "isShader", 1, is_shader)?;
    method(scope, prototype, "isSync", 1, is_sync)?;
    method(scope, prototype, "isTexture", 1, is_texture)?;
    method(
        scope,
        prototype,
        "isTransformFeedback",
        1,
        is_transform_feedback,
    )?;
    method(scope, prototype, "isVertexArray", 1, is_vertex_array)?;
    method(scope, prototype, "lineWidth", 1, operation)?;
    method(scope, prototype, "linkProgram", 1, operation)?;
    method(
        scope,
        prototype,
        "pauseTransformFeedback",
        0,
        pause_transform_feedback,
    )?;
    method(scope, prototype, "pixelStorei", 2, operation)?;
    method(scope, prototype, "polygonOffset", 2, operation)?;
    method(scope, prototype, "readBuffer", 1, read_buffer)?;
    method(scope, prototype, "readPixels", 7, operation)?;
    method(scope, prototype, "renderbufferStorage", 4, operation)?;
    method(
        scope,
        prototype,
        "renderbufferStorageMultisample",
        5,
        operation,
    )?;
    method(
        scope,
        prototype,
        "resumeTransformFeedback",
        0,
        resume_transform_feedback,
    )?;
    method(scope, prototype, "sampleCoverage", 2, operation)?;
    method(scope, prototype, "samplerParameterf", 3, sampler_parameter)?;
    method(scope, prototype, "samplerParameteri", 3, sampler_parameter)?;
    method(scope, prototype, "shaderSource", 2, operation)?;
    method(scope, prototype, "stencilFunc", 3, operation)?;
    method(scope, prototype, "stencilFuncSeparate", 4, operation)?;
    method(scope, prototype, "stencilMask", 1, operation)?;
    method(scope, prototype, "stencilMaskSeparate", 2, operation)?;
    method(scope, prototype, "stencilOp", 3, operation)?;
    method(scope, prototype, "stencilOpSeparate", 4, operation)?;
    method(scope, prototype, "texImage2D", 6, operation)?;
    method(scope, prototype, "texImage3D", 10, operation)?;
    method(scope, prototype, "texParameterf", 3, operation)?;
    method(scope, prototype, "texParameteri", 3, operation)?;
    method(scope, prototype, "texStorage2D", 5, operation)?;
    method(scope, prototype, "texStorage3D", 6, operation)?;
    method(scope, prototype, "texSubImage2D", 7, operation)?;
    method(scope, prototype, "texSubImage3D", 11, operation)?;
    method(scope, prototype, "transformFeedbackVaryings", 3, operation)?;
    method(scope, prototype, "uniform1ui", 2, operation)?;
    method(scope, prototype, "uniform2ui", 3, operation)?;
    method(scope, prototype, "uniform3ui", 4, operation)?;
    method(scope, prototype, "uniform4ui", 5, operation)?;
    method(scope, prototype, "uniformBlockBinding", 3, operation)?;
    method(scope, prototype, "useProgram", 1, operation)?;
    method(scope, prototype, "validateProgram", 1, operation)?;
    method(scope, prototype, "vertexAttribDivisor", 2, operation)?;
    method(scope, prototype, "vertexAttribI4i", 5, operation)?;
    method(scope, prototype, "vertexAttribI4ui", 5, operation)?;
    method(scope, prototype, "vertexAttribIPointer", 5, operation)?;
    method(scope, prototype, "waitSync", 3, wait_sync)?;
    method(scope, prototype, "bindBuffer", 2, operation)?;
    method(scope, prototype, "bindFramebuffer", 2, operation)?;
    method(scope, prototype, "bindTexture", 2, operation)?;
    method(scope, prototype, "clear", 1, operation)?;
    method(scope, prototype, "clearBufferfi", 4, operation)?;
    method(scope, prototype, "clearBufferfv", 3, operation)?;
    method(scope, prototype, "clearBufferiv", 3, operation)?;
    method(scope, prototype, "clearBufferuiv", 3, operation)?;
    method(scope, prototype, "clearColor", 4, operation)?;
    method(scope, prototype, "clearDepth", 1, operation)?;
    method(scope, prototype, "clearStencil", 1, operation)?;
    method(scope, prototype, "colorMask", 4, operation)?;
    method(scope, prototype, "disableVertexAttribArray", 1, operation)?;
    method(scope, prototype, "drawArrays", 3, operation)?;
    method(scope, prototype, "drawBuffers", 1, draw_buffers)?;
    method(scope, prototype, "drawElements", 4, operation)?;
    method(scope, prototype, "enableVertexAttribArray", 1, operation)?;
    method(scope, prototype, "scissor", 4, operation)?;
    method(scope, prototype, "uniform1f", 2, operation)?;
    method(scope, prototype, "uniform1fv", 2, operation)?;
    method(scope, prototype, "uniform1i", 2, operation)?;
    method(scope, prototype, "uniform1iv", 2, operation)?;
    method(scope, prototype, "uniform1uiv", 2, operation)?;
    method(scope, prototype, "uniform2f", 3, operation)?;
    method(scope, prototype, "uniform2fv", 2, operation)?;
    method(scope, prototype, "uniform2i", 3, operation)?;
    method(scope, prototype, "uniform2iv", 2, operation)?;
    method(scope, prototype, "uniform2uiv", 2, operation)?;
    method(scope, prototype, "uniform3f", 4, operation)?;
    method(scope, prototype, "uniform3fv", 2, operation)?;
    method(scope, prototype, "uniform3i", 4, operation)?;
    method(scope, prototype, "uniform3iv", 2, operation)?;
    method(scope, prototype, "uniform3uiv", 2, operation)?;
    method(scope, prototype, "uniform4f", 5, operation)?;
    method(scope, prototype, "uniform4fv", 2, operation)?;
    method(scope, prototype, "uniform4i", 5, operation)?;
    method(scope, prototype, "uniform4iv", 2, operation)?;
    method(scope, prototype, "uniform4uiv", 2, operation)?;
    method(scope, prototype, "uniformMatrix2fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix2x3fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix2x4fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix3fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix3x2fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix3x4fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix4fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix4x2fv", 3, operation)?;
    method(scope, prototype, "uniformMatrix4x3fv", 3, operation)?;
    method(scope, prototype, "vertexAttrib1f", 2, operation)?;
    method(scope, prototype, "vertexAttrib1fv", 2, operation)?;
    method(scope, prototype, "vertexAttrib2f", 3, operation)?;
    method(scope, prototype, "vertexAttrib2fv", 2, operation)?;
    method(scope, prototype, "vertexAttrib3f", 4, operation)?;
    method(scope, prototype, "vertexAttrib3fv", 2, operation)?;
    method(scope, prototype, "vertexAttrib4f", 5, operation)?;
    method(scope, prototype, "vertexAttrib4fv", 2, operation)?;
    method(scope, prototype, "vertexAttribI4iv", 2, operation)?;
    method(scope, prototype, "vertexAttribI4uiv", 2, operation)?;
    method(scope, prototype, "vertexAttribPointer", 6, operation)?;
    method(scope, prototype, "viewport", 4, operation)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "drawingBufferFormat",
        get_drawing_buffer_format,
    )?;
    method(
        scope,
        prototype,
        "drawingBufferStorage",
        3,
        drawing_buffer_storage,
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    canvas: Option<v8::Local<'_, v8::Object>>,
    width: u32,
    height: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let context = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, context, prototype.into()) != Some(true) {
        return Err("cannot create WebGL2RenderingContext".to_owned());
    }
    let canvas = canvas.map(|canvas| v8::Global::new(scope, canvas));
    scope
        .get_slot_mut::<WebGl2RenderingContextStore>()
        .ok_or_else(|| "WebGL2RenderingContext state was not prepared".to_owned())?
        .records
        .insert(
            context.get_identity_hash().get(),
            ContextRecord {
                canvas,
                width,
                height,
                drawing_buffer_color_space: "srgb".to_owned(),
                unpack_color_space: "srgb".to_owned(),
                error: NO_ERROR,
                buffers: HashSet::new(),
                framebuffers: HashSet::new(),
                programs: HashSet::new(),
                renderbuffers: HashSet::new(),
                shaders: HashSet::new(),
                textures: HashSet::new(),
                queries: HashMap::new(),
                samplers: HashMap::new(),
                syncs: HashMap::new(),
                transform_feedbacks: HashMap::new(),
                vertex_arrays: HashSet::new(),
                active_queries: HashMap::new(),
                bound_samplers: HashMap::new(),
                bound_transform_feedback: None,
                bound_vertex_array: None,
                read_buffer: 0x0405,
                draw_buffers: vec![0x0405],
                operation_count: 0,
                extensions: HashMap::new(),
            },
        );
    Ok(context)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ContextRecord> {
    scope
        .get_slot::<WebGl2RenderingContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut ContextRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<WebGl2RenderingContextStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    operation(record);
    true
}

fn set_error(record: &mut ContextRecord, error: u32) {
    if record.error == NO_ERROR {
        record.error = error;
    }
}

fn object_id(value: v8::Local<'_, v8::Value>) -> Option<i32> {
    v8::Local::<v8::Object>::try_from(value)
        .ok()
        .map(|object| object.get_identity_hash().get())
}

fn get_canvas(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(canvas) = record.canvas {
        result.set(v8::Local::new(scope, &canvas).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_drawing_buffer_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_drawing_buffer_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.height).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_string(scope: &v8::PinScope<'_, '_>, result: &mut v8::ReturnValue<'_>, value: &str) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn get_drawing_buffer_color_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.drawing_buffer_color_space);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_drawing_buffer_color_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        record.drawing_buffer_color_space = value
    });
}

fn get_unpack_color_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.unpack_color_space);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_unpack_color_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        record.unpack_color_space = value
    });
}

fn get_drawing_buffer_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0x8058).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn drawing_buffer_storage(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let height = arguments.get(2).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| {
        record.width = width;
        record.height = height;
        record.operation_count = record.operation_count.saturating_add(1);
    });
}

fn operation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.operation_count = record.operation_count.saturating_add(1);
    });
}

fn return_null(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_false(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, false).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_zero(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_negative_one(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, -1).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_invalid_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new_from_unsigned(scope, u32::MAX).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_empty_array(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Array::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_empty_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        return_string(scope, &mut result, "");
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn create_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_buffer::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.buffers.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_framebuffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_framebuffer::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.framebuffers.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_program(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_program::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.programs.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_renderbuffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_renderbuffer::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.renderbuffers.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_shader(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let kind = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if kind != 0x8B30 && kind != 0x8B31 {
        update(scope, arguments.this(), |record| {
            set_error(record, INVALID_ENUM)
        });
        result.set(v8::null(scope).into());
        return;
    }
    match super::webgl_shader::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.shaders.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_texture::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.textures.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_query::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            let stored_object = v8::Global::new(scope, object);
            if update(scope, arguments.this(), |record| {
                record.queries.insert(
                    id,
                    QueryRecord {
                        object: stored_object,
                        target: 0,
                        active: false,
                        available: false,
                        result: 0,
                    },
                );
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_sampler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_sampler::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.samplers.insert(id, SamplerRecord::default());
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_transform_feedback::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record
                    .transform_feedbacks
                    .insert(id, TransformFeedbackRecord::default());
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn create_vertex_array(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::webgl_vertex_array_object::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            if update(scope, arguments.this(), |record| {
                record.vertex_arrays.insert(id);
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn delete_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.buffers.remove(&id);
        });
    }
}
fn delete_framebuffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.framebuffers.remove(&id);
        });
    }
}
fn delete_program(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.programs.remove(&id);
        });
    }
}
fn delete_renderbuffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.renderbuffers.remove(&id);
        });
    }
}
fn delete_shader(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.shaders.remove(&id);
        });
    }
}
fn delete_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.textures.remove(&id);
        });
    }
}
fn delete_query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.queries.remove(&id);
            record.active_queries.retain(|_, query| *query != id);
        });
    }
}
fn delete_sampler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.samplers.remove(&id);
            record.bound_samplers.retain(|_, sampler| *sampler != id);
        });
    }
}
fn delete_sync(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.syncs.remove(&id);
        });
    }
}
fn delete_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.transform_feedbacks.remove(&id);
            if record.bound_transform_feedback == Some(id) {
                record.bound_transform_feedback = None
            }
        });
    }
}
fn delete_vertex_array(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(arguments.get(0)) {
        update(scope, arguments.this(), |record| {
            record.vertex_arrays.remove(&id);
            if record.bound_vertex_array == Some(id) {
                record.bound_vertex_array = None
            }
        });
    }
}

fn membership(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    test: impl FnOnce(&ContextRecord, i32) -> bool,
) {
    let id = object_id(arguments.get(0));
    let value = record(scope, arguments.this())
        .zip(id)
        .is_some_and(|(record, id)| test(&record, id));
    result.set(v8::Boolean::new(scope, value).into());
}

fn is_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.buffers.contains(&id))
}
fn is_framebuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.framebuffers.contains(&id))
}
fn is_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.programs.contains(&id))
}
fn is_query(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.queries.contains_key(&id))
}
fn is_renderbuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.renderbuffers.contains(&id))
}
fn is_sampler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.samplers.contains_key(&id))
}
fn is_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.shaders.contains(&id))
}
fn is_sync(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.syncs.contains_key(&id))
}
fn is_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.textures.contains(&id))
}
fn is_transform_feedback(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| {
        record.transform_feedbacks.contains_key(&id)
    })
}
fn is_vertex_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    membership(s, a, r, |record, id| record.vertex_arrays.contains(&id))
}

fn begin_query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let query = object_id(arguments.get(1));
    update(scope, arguments.this(), |record| {
        if record.active_queries.contains_key(&target) {
            set_error(record, INVALID_OPERATION);
            return;
        }
        let Some(query) = query.and_then(|id| record.queries.get_mut(&id).map(|query| (id, query)))
        else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        query.1.target = target;
        query.1.active = true;
        query.1.available = false;
        record.active_queries.insert(target, query.0);
    });
}

fn end_query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| {
        let Some(query_id) = record.active_queries.remove(&target) else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        if let Some(query) = record.queries.get_mut(&query_id) {
            query.active = false;
            query.available = true;
            query.result = record.operation_count.min(u32::MAX as u64) as u32;
        }
    });
}

fn get_query(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let target = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let query = record(scope, arguments.this())
        .and_then(|record| record.active_queries.get(&target).copied())
        .and_then(|id| {
            record(scope, arguments.this())?
                .queries
                .get(&id)
                .map(|query| query.object.clone())
        });
    if let Some(query) = query {
        result.set(v8::Local::new(scope, &query).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_query_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let query = object_id(arguments.get(0));
    let parameter = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let query = query.and_then(|id| record(scope, arguments.this())?.queries.get(&id).cloned());
    let Some(query) = query else {
        result.set(v8::null(scope).into());
        return;
    };
    match parameter {
        0x8867 => result.set(v8::Boolean::new(scope, query.available).into()),
        0x8866 => result.set(v8::Integer::new_from_unsigned(scope, query.result).into()),
        _ => result.set(v8::null(scope).into()),
    }
}

fn bind_sampler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let unit = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let sampler = if arguments.get(1).is_null() {
        None
    } else {
        object_id(arguments.get(1))
    };
    update(scope, arguments.this(), |record| match sampler {
        Some(sampler) if record.samplers.contains_key(&sampler) => {
            record.bound_samplers.insert(unit, sampler);
        }
        None => {
            record.bound_samplers.remove(&unit);
        }
        _ => set_error(record, INVALID_OPERATION),
    });
}

fn sampler_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let sampler = object_id(arguments.get(0));
    let parameter = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let value = arguments.get(2).number_value(scope).unwrap_or(0.0);
    update(scope, arguments.this(), |record| {
        let Some(sampler) = sampler.and_then(|id| record.samplers.get_mut(&id)) else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        sampler.parameters.insert(parameter, value);
    });
}

fn get_sampler_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let sampler = object_id(arguments.get(0));
    let parameter = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let value = sampler.and_then(|id| {
        record(scope, arguments.this())?
            .samplers
            .get(&id)?
            .parameters
            .get(&parameter)
            .copied()
    });
    if let Some(value) = value {
        result.set(v8::Number::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn bind_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let feedback = if arguments.get(1).is_null() {
        None
    } else {
        object_id(arguments.get(1))
    };
    update(scope, arguments.this(), |record| {
        if target != 0x8E22 {
            set_error(record, INVALID_ENUM);
        } else if feedback.is_none()
            || feedback.is_some_and(|id| record.transform_feedbacks.contains_key(&id))
        {
            record.bound_transform_feedback = feedback;
        } else {
            set_error(record, INVALID_OPERATION);
        }
    });
}

fn begin_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        let Some(feedback) = record
            .bound_transform_feedback
            .and_then(|id| record.transform_feedbacks.get_mut(&id))
        else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        feedback.active = true;
        feedback.paused = false;
    });
}

fn end_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        let Some(feedback) = record
            .bound_transform_feedback
            .and_then(|id| record.transform_feedbacks.get_mut(&id))
        else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        feedback.active = false;
        feedback.paused = false;
    });
}

fn pause_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        let Some(feedback) = record
            .bound_transform_feedback
            .and_then(|id| record.transform_feedbacks.get_mut(&id))
        else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        if feedback.active {
            feedback.paused = true;
        } else {
            set_error(record, INVALID_OPERATION);
        }
    });
}

fn resume_transform_feedback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        let Some(feedback) = record
            .bound_transform_feedback
            .and_then(|id| record.transform_feedbacks.get_mut(&id))
        else {
            set_error(record, INVALID_OPERATION);
            return;
        };
        if feedback.active && feedback.paused {
            feedback.paused = false;
        } else {
            set_error(record, INVALID_OPERATION);
        }
    });
}

fn bind_vertex_array(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let vertex_array = if arguments.get(0).is_null() {
        None
    } else {
        object_id(arguments.get(0))
    };
    update(scope, arguments.this(), |record| {
        if vertex_array.is_none()
            || vertex_array.is_some_and(|id| record.vertex_arrays.contains(&id))
        {
            record.bound_vertex_array = vertex_array;
        } else {
            set_error(record, INVALID_OPERATION);
        }
    });
}

fn fence_sync(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let condition = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let flags = arguments.get(1).uint32_value(scope).unwrap_or(0);
    if condition != 0x9117 || flags != 0 {
        update(scope, arguments.this(), |record| {
            set_error(record, INVALID_VALUE)
        });
        result.set(v8::null(scope).into());
        return;
    }
    match super::webgl_sync::create(scope) {
        Ok(object) => {
            let id = object.get_identity_hash().get();
            let stored = v8::Global::new(scope, object);
            if update(scope, arguments.this(), |record| {
                record.syncs.insert(
                    id,
                    SyncRecord {
                        object: stored,
                        condition,
                        flags,
                        signaled: false,
                    },
                );
            }) {
                result.set(object.into());
            }
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn client_wait_sync(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let sync = object_id(arguments.get(0));
    let value = sync.and_then(|id| {
        let mut status = 0x911B;
        update(scope, arguments.this(), |record| {
            if let Some(sync) = record.syncs.get_mut(&id) {
                if sync.signaled {
                    status = 0x911A;
                } else {
                    sync.signaled = true;
                    status = 0x911C;
                }
            } else {
                status = 0x911D;
                set_error(record, INVALID_VALUE);
            }
        });
        Some(status)
    });
    result.set(v8::Integer::new_from_unsigned(scope, value.unwrap_or(0x911D)).into());
}

fn wait_sync(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let sync = object_id(arguments.get(0));
    update(scope, arguments.this(), |record| {
        let Some(sync) = sync.and_then(|id| record.syncs.get_mut(&id)) else {
            set_error(record, INVALID_VALUE);
            return;
        };
        sync.signaled = true;
    });
}

fn get_sync_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let sync = object_id(arguments.get(0));
    let parameter = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let sync = sync.and_then(|id| record(scope, arguments.this())?.syncs.get(&id).cloned());
    let Some(sync) = sync else {
        result.set(v8::null(scope).into());
        return;
    };
    let value = match parameter {
        0x9112 => 0x9116,
        0x9113 => sync.condition,
        0x9114 => {
            if sync.signaled {
                0x9119
            } else {
                0x9118
            }
        }
        0x9115 => sync.flags,
        _ => {
            result.set(v8::null(scope).into());
            return;
        }
    };
    result.set(v8::Integer::new_from_unsigned(scope, value).into());
}

fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let error = record(scope, arguments.this())
        .map(|record| record.error)
        .unwrap_or(NO_ERROR);
    update(scope, arguments.this(), |record| record.error = NO_ERROR);
    result.set(v8::Integer::new_from_unsigned(scope, error).into());
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

fn get_context_attributes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let configured = crate::fingerprint::edge(scope).rendering.webgl.clone();
    let attributes = v8::Object::new(scope);
    let alpha = v8::Boolean::new(scope, configured.context_alpha);
    define_data(scope, attributes, "alpha", alpha.into());
    let depth = v8::Boolean::new(scope, configured.context_depth);
    define_data(scope, attributes, "depth", depth.into());
    let stencil = v8::Boolean::new(scope, configured.context_stencil);
    define_data(scope, attributes, "stencil", stencil.into());
    let antialias = v8::Boolean::new(scope, configured.context_antialias);
    define_data(scope, attributes, "antialias", antialias.into());
    let desynchronized = v8::Boolean::new(scope, configured.context_desynchronized);
    define_data(scope, attributes, "desynchronized", desynchronized.into());
    let caveat = v8::Boolean::new(scope, configured.context_fail_if_major_performance_caveat);
    define_data(
        scope,
        attributes,
        "failIfMajorPerformanceCaveat",
        caveat.into(),
    );
    if let Some(power_preference) = v8::String::new(scope, &configured.context_power_preference) {
        define_data(
            scope,
            attributes,
            "powerPreference",
            power_preference.into(),
        );
    }
    let premultiplied = v8::Boolean::new(scope, configured.context_premultiplied_alpha);
    define_data(
        scope,
        attributes,
        "premultipliedAlpha",
        premultiplied.into(),
    );
    let preserve = v8::Boolean::new(scope, configured.context_preserve_drawing_buffer);
    define_data(scope, attributes, "preserveDrawingBuffer", preserve.into());
    let xr_compatible = v8::Boolean::new(scope, configured.context_xr_compatible);
    define_data(scope, attributes, "xrCompatible", xr_compatible.into());
    result.set(attributes.into());
}

fn get_internalformat_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let array = v8::Array::new(scope, 4);
    let one = v8::Integer::new(scope, 1);
    let two = v8::Integer::new(scope, 2);
    let four = v8::Integer::new(scope, 4);
    let eight = v8::Integer::new(scope, 8);
    let _ = array.set_index(scope, 0, one.into());
    let _ = array.set_index(scope, 1, two.into());
    let _ = array.set_index(scope, 2, four.into());
    let _ = array.set_index(scope, 3, eight.into());
    result.set(array.into());
}

fn get_shader_precision_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let configured = crate::fingerprint::edge(scope).rendering.webgl.clone();
    match super::webgl_shader_precision_format::create(
        scope,
        configured.shader_precision_range_min,
        configured.shader_precision_range_max,
        configured.shader_precision_bits,
    ) {
        Ok(format) => result.set(format.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn get_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let parameter = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match parameter {
        0x0C02 => result.set(v8::Integer::new_from_unsigned(scope, record.read_buffer).into()),
        0x85B5 => result.set(v8::null(scope).into()),
        0x8E25 => result.set(v8::null(scope).into()),
        0x8E23 => result.set(v8::null(scope).into()),
        0x8824 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_draw_buffers;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8CDF => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_color_attachments;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8D57 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_samples;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8073 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_3d_texture_size;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x88FF => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_array_texture_layers;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8B4A => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_vertex_uniform_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8B49 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_fragment_uniform_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8B4B => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_varying_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x9122 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_vertex_output_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x9125 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_fragment_input_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A2B => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_vertex_uniform_blocks;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A2D => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_fragment_uniform_blocks;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A2E => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_combined_uniform_blocks;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A2F => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_uniform_buffer_bindings;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A30 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_uniform_block_size;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A31 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_combined_vertex_uniform_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8A33 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_combined_fragment_uniform_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8C8B => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_transform_feedback_separate_attribs;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8C8A => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_transform_feedback_interleaved_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8C80 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_transform_feedback_separate_components;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8905 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_program_texel_offset;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x80E8 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_elements_vertices;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x80E9 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_elements_indices;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8D6B => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_element_index;
            result.set(v8::Integer::new_from_unsigned(scope, value).into())
        }
        0x84FD => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_max_texture_lod_bias;
            result.set(v8::Number::new(scope, value).into())
        }
        0x821B => result.set(v8::Integer::new(scope, 2).into()),
        0x821C => result.set(v8::Integer::new(scope, 0).into()),
        0x1F00 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .vendor
                .clone();
            return_string(scope, &mut result, &value)
        }
        0x1F01 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .renderer
                .clone();
            return_string(scope, &mut result, &value)
        }
        0x9245 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .unmasked_vendor
                .clone();
            return_string(scope, &mut result, &value)
        }
        0x9246 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .unmasked_renderer
                .clone();
            return_string(scope, &mut result, &value)
        }
        0x84FF => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_anisotropy;
            result.set(v8::Number::new(scope, value).into())
        }
        0x0D33 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_texture_size;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x851C => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_cube_map_texture_size;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x84E8 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_renderbuffer_size;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8869 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_vertex_attribs;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8DFB => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_vertex_uniform_vectors;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8DFC => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_varying_vectors;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8DFD => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_fragment_uniform_vectors;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8B4C => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_vertex_texture_image_units;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8872 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_texture_image_units;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x8B4D => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .max_combined_texture_image_units;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x0D50 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .subpixel_bits;
            result.set(v8::Integer::new(scope, value).into())
        }
        0x0B93 | 0x0B98 | 0x8CA4 | 0x8CA5 => {
            result.set(v8::Integer::new_from_unsigned(scope, u32::MAX).into())
        }
        0x846D => {
            let configured = &crate::fingerprint::edge(scope).rendering.webgl;
            return_float32_array(
                scope,
                &mut result,
                &[
                    configured.aliased_point_size_min,
                    configured.aliased_point_size_max,
                ],
            )
        }
        0x846E => {
            let configured = &crate::fingerprint::edge(scope).rendering.webgl;
            return_float32_array(
                scope,
                &mut result,
                &[
                    configured.aliased_line_width_min,
                    configured.aliased_line_width_max,
                ],
            )
        }
        0x0D3A => {
            let configured = &crate::fingerprint::edge(scope).rendering.webgl;
            return_int32_array(
                scope,
                &mut result,
                &[
                    configured.max_viewport_width,
                    configured.max_viewport_height,
                ],
            )
        }
        0x86A3 => {
            let configured = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .compressed_texture_formats
                .clone();
            let values = v8::Array::new(scope, configured.len() as i32);
            for (index, format) in configured.into_iter().enumerate() {
                let value = v8::Integer::new_from_unsigned(scope, format);
                let _ = values.set_index(scope, index as u32, value.into());
            }
            result.set(values.into())
        }
        0x1F02 => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_version
                .clone();
            return_string(scope, &mut result, &value)
        }
        0x8B8C => {
            let value = crate::fingerprint::edge(scope)
                .rendering
                .webgl
                .webgl2_shading_language_version
                .clone();
            return_string(scope, &mut result, &value)
        }
        _ => {
            update(scope, arguments.this(), |context| {
                set_error(context, INVALID_ENUM)
            });
            result.set(v8::null(scope).into())
        }
    }
}

fn return_float32_array(
    scope: &v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    values: &[f64],
) {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&(*value as f32).to_ne_bytes());
    }
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    if let Some(array) = v8::Float32Array::new(scope, buffer, 0, values.len()) {
        result.set(array.into());
    }
}

fn return_int32_array(
    scope: &v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    values: &[i32],
) {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    if let Some(array) = v8::Int32Array::new(scope, buffer, 0, values.len()) {
        result.set(array.into());
    }
}

fn get_supported_extensions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let extensions = crate::fingerprint::edge(scope)
        .rendering
        .webgl
        .webgl2_extensions
        .clone();
    let array = v8::Array::new(scope, extensions.len() as i32);
    for (index, extension) in extensions.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, extension) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    result.set(array.into());
}

fn get_extension(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(context) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let requested = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = crate::fingerprint::edge(scope)
        .rendering
        .webgl
        .webgl2_extensions
        .iter()
        .find(|name| name.eq_ignore_ascii_case(&requested))
        .cloned();
    let Some(name) = supported else {
        result.set(v8::null(scope).into());
        return;
    };
    if let Some(existing) = context.extensions.get(&name) {
        result.set(v8::Local::new(scope, existing).into());
        return;
    }
    let extension = v8::Object::new(scope);
    if name.eq_ignore_ascii_case("WEBGL_debug_renderer_info") {
        define_data(
            scope,
            extension,
            "UNMASKED_VENDOR_WEBGL",
            v8::Integer::new(scope, 0x9245).into(),
        );
        define_data(
            scope,
            extension,
            "UNMASKED_RENDERER_WEBGL",
            v8::Integer::new(scope, 0x9246).into(),
        );
    }
    let stored = v8::Global::new(scope, extension);
    update(scope, arguments.this(), |context| {
        context.extensions.insert(name, stored);
    });
    result.set(extension.into());
}

fn check_framebuffer_status(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new_from_unsigned(scope, 0x8CD5).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn read_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mode = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| record.read_buffer = mode);
}

fn numeric_values(scope: &mut v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Vec<u32> {
    let Some(object) = v8::Local::<v8::Object>::try_from(value).ok() else {
        return Vec::new();
    };
    let length = v8::String::new(scope, "length")
        .and_then(|key| object.get(scope, key.into()))
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut values = Vec::with_capacity(length as usize);
    for index in 0..length {
        values.push(
            object
                .get_index(scope, index)
                .and_then(|value| value.uint32_value(scope))
                .unwrap_or(0),
        );
    }
    values
}

fn draw_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let buffers = numeric_values(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        if buffers.len() > 16 {
            set_error(record, INVALID_VALUE);
        } else {
            record.draw_buffers = buffers;
            record.operation_count = record.operation_count.saturating_add(1);
        }
    });
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebGl2RenderingContextStore>() {
        store.constructors.remove(&realm_id);
    }
}
