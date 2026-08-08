use std::collections::{HashMap, HashSet};

const NO_ERROR: u32 = 0;
const INVALID_ENUM: u32 = 0x0500;
const INVALID_VALUE: u32 = 0x0501;
const INVALID_OPERATION: u32 = 0x0502;
const ARRAY_BUFFER: u32 = 0x8892;
const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const BUFFER_SIZE: u32 = 0x8764;
const BUFFER_USAGE: u32 = 0x8765;
const FRAGMENT_SHADER: u32 = 0x8B30;
const VERTEX_SHADER: u32 = 0x8B31;
const SHADER_TYPE: u32 = 0x8B4F;
const DELETE_STATUS: u32 = 0x8B80;
const COMPILE_STATUS: u32 = 0x8B81;
const LINK_STATUS: u32 = 0x8B82;
const VALIDATE_STATUS: u32 = 0x8B83;
const ATTACHED_SHADERS: u32 = 0x8B85;
const ACTIVE_UNIFORMS: u32 = 0x8B86;
const ACTIVE_ATTRIBUTES: u32 = 0x8B89;
const CURRENT_PROGRAM: u32 = 0x8B8D;
const FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;

#[derive(Default)]
pub(crate) struct WebGlRenderingContextStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ContextRecord>,
}

#[derive(Clone)]
struct BufferRecord {
    size: u32,
    usage: u32,
    deleted: bool,
}

#[derive(Clone)]
struct ShaderRecord {
    object: v8::Global<v8::Object>,
    kind: u32,
    source: String,
    compiled: bool,
    deleted: bool,
    info_log: String,
}

#[derive(Clone)]
struct ProgramRecord {
    attached_shaders: Vec<i32>,
    bound_attributes: HashMap<String, u32>,
    active_attributes: Vec<String>,
    active_uniforms: Vec<String>,
    linked: bool,
    validated: bool,
    deleted: bool,
    info_log: String,
}

#[derive(Clone, Default)]
struct TextureRecord {
    parameters: HashMap<u32, f64>,
    width: u32,
    height: u32,
    format: u32,
    mipmaps_generated: bool,
}

#[derive(Clone, Default)]
struct RenderbufferRecord {
    width: u32,
    height: u32,
    internal_format: u32,
}

#[derive(Clone, Default)]
struct FramebufferRecord {
    attachments: HashMap<u32, i32>,
}

#[derive(Clone)]
struct UniformRecord {
    object: v8::Global<v8::Object>,
    program: i32,
    name: String,
    values: Vec<f64>,
}

#[derive(Clone)]
struct VertexAttribRecord {
    size: i32,
    kind: u32,
    normalized: bool,
    stride: i32,
    offset: i32,
    buffer: Option<v8::Global<v8::Object>>,
}

#[derive(Clone)]
struct ContextRecord {
    canvas: Option<v8::Global<v8::Object>>,
    width: u32,
    height: u32,
    drawing_buffer_color_space: String,
    unpack_color_space: String,
    error: u32,
    buffers: HashMap<i32, BufferRecord>,
    shaders: HashMap<i32, ShaderRecord>,
    programs: HashMap<i32, ProgramRecord>,
    framebuffers: HashMap<i32, FramebufferRecord>,
    renderbuffers: HashMap<i32, RenderbufferRecord>,
    textures: HashMap<i32, TextureRecord>,
    uniform_locations: HashMap<i32, UniformRecord>,
    bound_array_buffer: Option<v8::Global<v8::Object>>,
    bound_element_array_buffer: Option<v8::Global<v8::Object>>,
    bound_framebuffer: Option<v8::Global<v8::Object>>,
    bound_renderbuffer: Option<v8::Global<v8::Object>>,
    bound_texture_2d: Option<v8::Global<v8::Object>>,
    bound_texture_cube_map: Option<v8::Global<v8::Object>>,
    current_program: Option<v8::Global<v8::Object>>,
    active_texture: u32,
    enabled_capabilities: HashSet<u32>,
    clear_color: [f64; 4],
    clear_depth: f64,
    clear_stencil: i32,
    color_mask: [bool; 4],
    depth_function: u32,
    depth_write_mask: bool,
    depth_range: [f64; 2],
    cull_face_mode: u32,
    front_face: u32,
    line_width: f64,
    blend_color: [f64; 4],
    blend_equation: [u32; 2],
    blend_function: [u32; 4],
    pixel_store: HashMap<u32, i32>,
    polygon_offset: [f64; 2],
    sample_coverage: [f64; 2],
    scissor: [i32; 4],
    vertex_attrib_enabled: HashSet<u32>,
    vertex_attrib_values: HashMap<u32, [f64; 4]>,
    vertex_attrib_pointers: HashMap<u32, VertexAttribRecord>,
    extensions: HashMap<String, v8::Global<v8::Object>>,
    hints: HashMap<u32, u32>,
    stencil_front: [u32; 7],
    stencil_back: [u32; 7],
    viewport: [i32; 4],
    draw_calls: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WebGlRenderingContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebGLRenderingContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<WebGlRenderingContextStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WebGLRenderingContext",
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
    define_core_constants(scope, prototype)?;

    crate::webidl::define_method(scope, prototype, "activeTexture", 1, active_texture)?;
    crate::webidl::define_method(scope, prototype, "attachShader", 2, attach_shader)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "bindAttribLocation",
        3,
        bind_attrib_location,
    )?;
    crate::webidl::define_method(scope, prototype, "bindRenderbuffer", 2, bind_renderbuffer)?;
    crate::webidl::define_method(scope, prototype, "blendColor", 4, blend_color)?;
    crate::webidl::define_method(scope, prototype, "blendEquation", 1, blend_equation)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "blendEquationSeparate",
        2,
        blend_equation_separate,
    )?;
    crate::webidl::define_method(scope, prototype, "blendFunc", 2, blend_func)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "blendFuncSeparate",
        4,
        blend_func_separate,
    )?;
    crate::webidl::define_method(scope, prototype, "bufferData", 3, buffer_data)?;
    crate::webidl::define_method(scope, prototype, "bufferSubData", 3, buffer_sub_data)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "checkFramebufferStatus",
        1,
        check_framebuffer_status,
    )?;
    crate::webidl::define_method(scope, prototype, "compileShader", 1, compile_shader)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "compressedTexImage2D",
        7,
        compressed_tex_image_2d,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "compressedTexSubImage2D",
        8,
        compressed_tex_sub_image_2d,
    )?;
    crate::webidl::define_method(scope, prototype, "copyTexImage2D", 8, copy_tex_image_2d)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "copyTexSubImage2D",
        8,
        copy_tex_sub_image_2d,
    )?;
    crate::webidl::define_method(scope, prototype, "createBuffer", 0, create_buffer)?;
    crate::webidl::define_method(scope, prototype, "createFramebuffer", 0, create_framebuffer)?;
    crate::webidl::define_method(scope, prototype, "createProgram", 0, create_program)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createRenderbuffer",
        0,
        create_renderbuffer,
    )?;
    crate::webidl::define_method(scope, prototype, "createShader", 1, create_shader)?;
    crate::webidl::define_method(scope, prototype, "createTexture", 0, create_texture)?;
    crate::webidl::define_method(scope, prototype, "cullFace", 1, cull_face)?;
    crate::webidl::define_method(scope, prototype, "deleteBuffer", 1, delete_buffer)?;
    crate::webidl::define_method(scope, prototype, "deleteFramebuffer", 1, delete_framebuffer)?;
    crate::webidl::define_method(scope, prototype, "deleteProgram", 1, delete_program)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "deleteRenderbuffer",
        1,
        delete_renderbuffer,
    )?;
    crate::webidl::define_method(scope, prototype, "deleteShader", 1, delete_shader)?;
    crate::webidl::define_method(scope, prototype, "deleteTexture", 1, delete_texture)?;
    crate::webidl::define_method(scope, prototype, "depthFunc", 1, depth_func)?;
    crate::webidl::define_method(scope, prototype, "depthMask", 1, depth_mask)?;
    crate::webidl::define_method(scope, prototype, "depthRange", 2, depth_range)?;
    crate::webidl::define_method(scope, prototype, "detachShader", 2, detach_shader)?;
    crate::webidl::define_method(scope, prototype, "disable", 1, disable)?;
    crate::webidl::define_method(scope, prototype, "enable", 1, enable)?;
    crate::webidl::define_method(scope, prototype, "finish", 0, finish)?;
    crate::webidl::define_method(scope, prototype, "flush", 0, flush)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "framebufferRenderbuffer",
        4,
        framebuffer_renderbuffer,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "framebufferTexture2D",
        5,
        framebuffer_texture_2d,
    )?;
    crate::webidl::define_method(scope, prototype, "frontFace", 1, front_face)?;
    crate::webidl::define_method(scope, prototype, "generateMipmap", 1, generate_mipmap)?;
    crate::webidl::define_method(scope, prototype, "getActiveAttrib", 2, get_active_attrib)?;
    crate::webidl::define_method(scope, prototype, "getActiveUniform", 2, get_active_uniform)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getAttachedShaders",
        1,
        get_attached_shaders,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getAttribLocation",
        2,
        get_attrib_location,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getBufferParameter",
        2,
        get_buffer_parameter,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getContextAttributes",
        0,
        get_context_attributes,
    )?;
    crate::webidl::define_method(scope, prototype, "getError", 0, get_error)?;
    crate::webidl::define_method(scope, prototype, "getExtension", 1, get_extension)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getFramebufferAttachmentParameter",
        3,
        get_framebuffer_attachment_parameter,
    )?;
    crate::webidl::define_method(scope, prototype, "getParameter", 1, get_parameter)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getProgramInfoLog",
        1,
        get_program_info_log,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getProgramParameter",
        2,
        get_program_parameter,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getRenderbufferParameter",
        2,
        get_renderbuffer_parameter,
    )?;
    crate::webidl::define_method(scope, prototype, "getShaderInfoLog", 1, get_shader_info_log)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getShaderParameter",
        2,
        get_shader_parameter,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getShaderPrecisionFormat",
        2,
        get_shader_precision_format,
    )?;
    crate::webidl::define_method(scope, prototype, "getShaderSource", 1, get_shader_source)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getSupportedExtensions",
        0,
        get_supported_extensions,
    )?;
    crate::webidl::define_method(scope, prototype, "getTexParameter", 2, get_tex_parameter)?;
    crate::webidl::define_method(scope, prototype, "getUniform", 2, get_uniform)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getUniformLocation",
        2,
        get_uniform_location,
    )?;
    crate::webidl::define_method(scope, prototype, "getVertexAttrib", 2, get_vertex_attrib)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getVertexAttribOffset",
        2,
        get_vertex_attrib_offset,
    )?;
    crate::webidl::define_method(scope, prototype, "hint", 2, hint)?;
    crate::webidl::define_method(scope, prototype, "isBuffer", 1, is_buffer)?;
    crate::webidl::define_method(scope, prototype, "isContextLost", 0, is_context_lost)?;
    crate::webidl::define_method(scope, prototype, "isEnabled", 1, is_enabled)?;
    crate::webidl::define_method(scope, prototype, "isFramebuffer", 1, is_framebuffer)?;
    crate::webidl::define_method(scope, prototype, "isProgram", 1, is_program)?;
    crate::webidl::define_method(scope, prototype, "isRenderbuffer", 1, is_renderbuffer)?;
    crate::webidl::define_method(scope, prototype, "isShader", 1, is_shader)?;
    crate::webidl::define_method(scope, prototype, "isTexture", 1, is_texture)?;
    crate::webidl::define_method(scope, prototype, "lineWidth", 1, line_width)?;
    crate::webidl::define_method(scope, prototype, "linkProgram", 1, link_program)?;
    crate::webidl::define_method(scope, prototype, "pixelStorei", 2, pixel_store_i)?;
    crate::webidl::define_method(scope, prototype, "polygonOffset", 2, polygon_offset)?;
    crate::webidl::define_method(scope, prototype, "readPixels", 7, read_pixels)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "renderbufferStorage",
        4,
        renderbuffer_storage,
    )?;
    crate::webidl::define_method(scope, prototype, "sampleCoverage", 2, sample_coverage)?;
    crate::webidl::define_method(scope, prototype, "shaderSource", 2, shader_source)?;
    crate::webidl::define_method(scope, prototype, "stencilFunc", 3, stencil_func)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "stencilFuncSeparate",
        4,
        stencil_func_separate,
    )?;
    crate::webidl::define_method(scope, prototype, "stencilMask", 1, stencil_mask)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "stencilMaskSeparate",
        2,
        stencil_mask_separate,
    )?;
    crate::webidl::define_method(scope, prototype, "stencilOp", 3, stencil_op)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "stencilOpSeparate",
        4,
        stencil_op_separate,
    )?;
    crate::webidl::define_method(scope, prototype, "texImage2D", 6, tex_image_2d)?;
    crate::webidl::define_method(scope, prototype, "texParameterf", 3, tex_parameter_f)?;
    crate::webidl::define_method(scope, prototype, "texParameteri", 3, tex_parameter_i)?;
    crate::webidl::define_method(scope, prototype, "texSubImage2D", 7, tex_sub_image_2d)?;
    crate::webidl::define_method(scope, prototype, "useProgram", 1, use_program)?;
    crate::webidl::define_method(scope, prototype, "validateProgram", 1, validate_program)?;
    crate::webidl::define_method(scope, prototype, "bindBuffer", 2, bind_buffer)?;
    crate::webidl::define_method(scope, prototype, "bindFramebuffer", 2, bind_framebuffer)?;
    crate::webidl::define_method(scope, prototype, "bindTexture", 2, bind_texture)?;
    crate::webidl::define_method(scope, prototype, "clear", 1, clear)?;
    crate::webidl::define_method(scope, prototype, "clearColor", 4, clear_color)?;
    crate::webidl::define_method(scope, prototype, "clearDepth", 1, clear_depth)?;
    crate::webidl::define_method(scope, prototype, "clearStencil", 1, clear_stencil)?;
    crate::webidl::define_method(scope, prototype, "colorMask", 4, color_mask)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "disableVertexAttribArray",
        1,
        disable_vertex_attrib_array,
    )?;
    crate::webidl::define_method(scope, prototype, "drawArrays", 3, draw_arrays)?;
    crate::webidl::define_method(scope, prototype, "drawElements", 4, draw_elements)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "enableVertexAttribArray",
        1,
        enable_vertex_attrib_array,
    )?;
    crate::webidl::define_method(scope, prototype, "scissor", 4, scissor)?;
    crate::webidl::define_method(scope, prototype, "uniform1f", 2, uniform_1f)?;
    crate::webidl::define_method(scope, prototype, "uniform1fv", 2, uniform_1fv)?;
    crate::webidl::define_method(scope, prototype, "uniform1i", 2, uniform_1i)?;
    crate::webidl::define_method(scope, prototype, "uniform1iv", 2, uniform_1iv)?;
    crate::webidl::define_method(scope, prototype, "uniform2f", 3, uniform_2f)?;
    crate::webidl::define_method(scope, prototype, "uniform2fv", 2, uniform_2fv)?;
    crate::webidl::define_method(scope, prototype, "uniform2i", 3, uniform_2i)?;
    crate::webidl::define_method(scope, prototype, "uniform2iv", 2, uniform_2iv)?;
    crate::webidl::define_method(scope, prototype, "uniform3f", 4, uniform_3f)?;
    crate::webidl::define_method(scope, prototype, "uniform3fv", 2, uniform_3fv)?;
    crate::webidl::define_method(scope, prototype, "uniform3i", 4, uniform_3i)?;
    crate::webidl::define_method(scope, prototype, "uniform3iv", 2, uniform_3iv)?;
    crate::webidl::define_method(scope, prototype, "uniform4f", 5, uniform_4f)?;
    crate::webidl::define_method(scope, prototype, "uniform4fv", 2, uniform_4fv)?;
    crate::webidl::define_method(scope, prototype, "uniform4i", 5, uniform_4i)?;
    crate::webidl::define_method(scope, prototype, "uniform4iv", 2, uniform_4iv)?;
    crate::webidl::define_method(scope, prototype, "uniformMatrix2fv", 3, uniform_matrix_2fv)?;
    crate::webidl::define_method(scope, prototype, "uniformMatrix3fv", 3, uniform_matrix_3fv)?;
    crate::webidl::define_method(scope, prototype, "uniformMatrix4fv", 3, uniform_matrix_4fv)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib1f", 2, vertex_attrib_1f)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib1fv", 2, vertex_attrib_1fv)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib2f", 3, vertex_attrib_2f)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib2fv", 2, vertex_attrib_2fv)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib3f", 4, vertex_attrib_3f)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib3fv", 2, vertex_attrib_3fv)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib4f", 5, vertex_attrib_4f)?;
    crate::webidl::define_method(scope, prototype, "vertexAttrib4fv", 2, vertex_attrib_4fv)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "vertexAttribPointer",
        6,
        vertex_attrib_pointer,
    )?;
    crate::webidl::define_method(scope, prototype, "viewport", 4, viewport)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "drawingBufferFormat",
        get_drawing_buffer_format,
    )?;
    crate::webidl::define_constant(scope, prototype, "RGB8", 0x8051)?;
    crate::webidl::define_constant(scope, prototype, "RGBA8", 0x8058)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "drawingBufferStorage",
        3,
        drawing_buffer_storage,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, prototype, "makeXRCompatible", 0, make_xr_compatible)?;
    define_core_constants(scope, constructor.into())?;
    crate::webidl::define_constant(scope, constructor.into(), "RGB8", 0x8051)?;
    crate::webidl::define_constant(scope, constructor.into(), "RGBA8", 0x8058)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WebGlRenderingContextStore>()
        .ok_or_else(|| "WebGLRenderingContext state was not prepared".to_owned())?
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

pub(crate) fn define_core_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    define_webgl_constants_5_through_100(scope, object)?;
    define_webgl_constants_101_through_193(scope, object)?;
    define_webgl_constants_194_through_300(scope, object)
}

fn constant(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, name, value)
}

fn define_webgl_constants_5_through_100(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    constant(scope, object, "DEPTH_BUFFER_BIT", 256)?;
    constant(scope, object, "STENCIL_BUFFER_BIT", 1024)?;
    constant(scope, object, "COLOR_BUFFER_BIT", 16384)?;
    constant(scope, object, "POINTS", 0)?;
    constant(scope, object, "LINES", 1)?;
    constant(scope, object, "LINE_LOOP", 2)?;
    constant(scope, object, "LINE_STRIP", 3)?;
    constant(scope, object, "TRIANGLES", 4)?;
    constant(scope, object, "TRIANGLE_STRIP", 5)?;
    constant(scope, object, "TRIANGLE_FAN", 6)?;
    constant(scope, object, "ZERO", 0)?;
    constant(scope, object, "ONE", 1)?;
    constant(scope, object, "SRC_COLOR", 768)?;
    constant(scope, object, "ONE_MINUS_SRC_COLOR", 769)?;
    constant(scope, object, "SRC_ALPHA", 770)?;
    constant(scope, object, "ONE_MINUS_SRC_ALPHA", 771)?;
    constant(scope, object, "DST_ALPHA", 772)?;
    constant(scope, object, "ONE_MINUS_DST_ALPHA", 773)?;
    constant(scope, object, "DST_COLOR", 774)?;
    constant(scope, object, "ONE_MINUS_DST_COLOR", 775)?;
    constant(scope, object, "SRC_ALPHA_SATURATE", 776)?;
    constant(scope, object, "FUNC_ADD", 32774)?;
    constant(scope, object, "BLEND_EQUATION", 32777)?;
    constant(scope, object, "BLEND_EQUATION_RGB", 32777)?;
    constant(scope, object, "BLEND_EQUATION_ALPHA", 34877)?;
    constant(scope, object, "FUNC_SUBTRACT", 32778)?;
    constant(scope, object, "FUNC_REVERSE_SUBTRACT", 32779)?;
    constant(scope, object, "BLEND_DST_RGB", 32968)?;
    constant(scope, object, "BLEND_SRC_RGB", 32969)?;
    constant(scope, object, "BLEND_DST_ALPHA", 32970)?;
    constant(scope, object, "BLEND_SRC_ALPHA", 32971)?;
    constant(scope, object, "CONSTANT_COLOR", 32769)?;
    constant(scope, object, "ONE_MINUS_CONSTANT_COLOR", 32770)?;
    constant(scope, object, "CONSTANT_ALPHA", 32771)?;
    constant(scope, object, "ONE_MINUS_CONSTANT_ALPHA", 32772)?;
    constant(scope, object, "BLEND_COLOR", 32773)?;
    constant(scope, object, "ARRAY_BUFFER", 34962)?;
    constant(scope, object, "ELEMENT_ARRAY_BUFFER", 34963)?;
    constant(scope, object, "ARRAY_BUFFER_BINDING", 34964)?;
    constant(scope, object, "ELEMENT_ARRAY_BUFFER_BINDING", 34965)?;
    constant(scope, object, "STREAM_DRAW", 35040)?;
    constant(scope, object, "STATIC_DRAW", 35044)?;
    constant(scope, object, "DYNAMIC_DRAW", 35048)?;
    constant(scope, object, "BUFFER_SIZE", 34660)?;
    constant(scope, object, "BUFFER_USAGE", 34661)?;
    constant(scope, object, "CURRENT_VERTEX_ATTRIB", 34342)?;
    constant(scope, object, "FRONT", 1028)?;
    constant(scope, object, "BACK", 1029)?;
    constant(scope, object, "FRONT_AND_BACK", 1032)?;
    constant(scope, object, "TEXTURE_2D", 3553)?;
    constant(scope, object, "CULL_FACE", 2884)?;
    constant(scope, object, "BLEND", 3042)?;
    constant(scope, object, "DITHER", 3024)?;
    constant(scope, object, "STENCIL_TEST", 2960)?;
    constant(scope, object, "DEPTH_TEST", 2929)?;
    constant(scope, object, "SCISSOR_TEST", 3089)?;
    constant(scope, object, "POLYGON_OFFSET_FILL", 32823)?;
    constant(scope, object, "SAMPLE_ALPHA_TO_COVERAGE", 32926)?;
    constant(scope, object, "SAMPLE_COVERAGE", 32928)?;
    constant(scope, object, "NO_ERROR", 0)?;
    constant(scope, object, "INVALID_ENUM", 1280)?;
    constant(scope, object, "INVALID_VALUE", 1281)?;
    constant(scope, object, "INVALID_OPERATION", 1282)?;
    constant(scope, object, "OUT_OF_MEMORY", 1285)?;
    constant(scope, object, "CW", 2304)?;
    constant(scope, object, "CCW", 2305)?;
    constant(scope, object, "LINE_WIDTH", 2849)?;
    constant(scope, object, "ALIASED_POINT_SIZE_RANGE", 33901)?;
    constant(scope, object, "ALIASED_LINE_WIDTH_RANGE", 33902)?;
    constant(scope, object, "CULL_FACE_MODE", 2885)?;
    constant(scope, object, "FRONT_FACE", 2886)?;
    constant(scope, object, "DEPTH_RANGE", 2928)?;
    constant(scope, object, "DEPTH_WRITEMASK", 2930)?;
    constant(scope, object, "DEPTH_CLEAR_VALUE", 2931)?;
    constant(scope, object, "DEPTH_FUNC", 2932)?;
    constant(scope, object, "STENCIL_CLEAR_VALUE", 2961)?;
    constant(scope, object, "STENCIL_FUNC", 2962)?;
    constant(scope, object, "STENCIL_FAIL", 2964)?;
    constant(scope, object, "STENCIL_PASS_DEPTH_FAIL", 2965)?;
    constant(scope, object, "STENCIL_PASS_DEPTH_PASS", 2966)?;
    constant(scope, object, "STENCIL_REF", 2967)?;
    constant(scope, object, "STENCIL_VALUE_MASK", 2963)?;
    constant(scope, object, "STENCIL_WRITEMASK", 2968)?;
    constant(scope, object, "STENCIL_BACK_FUNC", 34816)?;
    constant(scope, object, "STENCIL_BACK_FAIL", 34817)?;
    constant(scope, object, "STENCIL_BACK_PASS_DEPTH_FAIL", 34818)?;
    constant(scope, object, "STENCIL_BACK_PASS_DEPTH_PASS", 34819)?;
    constant(scope, object, "STENCIL_BACK_REF", 36003)?;
    constant(scope, object, "STENCIL_BACK_VALUE_MASK", 36004)?;
    constant(scope, object, "STENCIL_BACK_WRITEMASK", 36005)?;
    constant(scope, object, "VIEWPORT", 2978)?;
    constant(scope, object, "SCISSOR_BOX", 3088)?;
    constant(scope, object, "COLOR_CLEAR_VALUE", 3106)?;
    constant(scope, object, "COLOR_WRITEMASK", 3107)?;
    constant(scope, object, "UNPACK_ALIGNMENT", 3317)?;
    constant(scope, object, "PACK_ALIGNMENT", 3333)
}

fn define_webgl_constants_101_through_193(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    constant(scope, object, "MAX_TEXTURE_SIZE", 3379)?;
    constant(scope, object, "MAX_VIEWPORT_DIMS", 3386)?;
    constant(scope, object, "SUBPIXEL_BITS", 3408)?;
    constant(scope, object, "RED_BITS", 3410)?;
    constant(scope, object, "GREEN_BITS", 3411)?;
    constant(scope, object, "BLUE_BITS", 3412)?;
    constant(scope, object, "ALPHA_BITS", 3413)?;
    constant(scope, object, "DEPTH_BITS", 3414)?;
    constant(scope, object, "STENCIL_BITS", 3415)?;
    constant(scope, object, "POLYGON_OFFSET_UNITS", 10752)?;
    constant(scope, object, "POLYGON_OFFSET_FACTOR", 32824)?;
    constant(scope, object, "TEXTURE_BINDING_2D", 32873)?;
    constant(scope, object, "SAMPLE_BUFFERS", 32936)?;
    constant(scope, object, "SAMPLES", 32937)?;
    constant(scope, object, "SAMPLE_COVERAGE_VALUE", 32938)?;
    constant(scope, object, "SAMPLE_COVERAGE_INVERT", 32939)?;
    constant(scope, object, "COMPRESSED_TEXTURE_FORMATS", 34467)?;
    constant(scope, object, "DONT_CARE", 4352)?;
    constant(scope, object, "FASTEST", 4353)?;
    constant(scope, object, "NICEST", 4354)?;
    constant(scope, object, "GENERATE_MIPMAP_HINT", 33170)?;
    constant(scope, object, "BYTE", 5120)?;
    constant(scope, object, "UNSIGNED_BYTE", 5121)?;
    constant(scope, object, "SHORT", 5122)?;
    constant(scope, object, "UNSIGNED_SHORT", 5123)?;
    constant(scope, object, "INT", 5124)?;
    constant(scope, object, "UNSIGNED_INT", 5125)?;
    constant(scope, object, "FLOAT", 5126)?;
    constant(scope, object, "DEPTH_COMPONENT", 6402)?;
    constant(scope, object, "ALPHA", 6406)?;
    constant(scope, object, "RGB", 6407)?;
    constant(scope, object, "RGBA", 6408)?;
    constant(scope, object, "LUMINANCE", 6409)?;
    constant(scope, object, "LUMINANCE_ALPHA", 6410)?;
    constant(scope, object, "UNSIGNED_SHORT_4_4_4_4", 32819)?;
    constant(scope, object, "UNSIGNED_SHORT_5_5_5_1", 32820)?;
    constant(scope, object, "UNSIGNED_SHORT_5_6_5", 33635)?;
    constant(scope, object, "FRAGMENT_SHADER", 35632)?;
    constant(scope, object, "VERTEX_SHADER", 35633)?;
    constant(scope, object, "MAX_VERTEX_ATTRIBS", 34921)?;
    constant(scope, object, "MAX_VERTEX_UNIFORM_VECTORS", 36347)?;
    constant(scope, object, "MAX_VARYING_VECTORS", 36348)?;
    constant(scope, object, "MAX_COMBINED_TEXTURE_IMAGE_UNITS", 35661)?;
    constant(scope, object, "MAX_VERTEX_TEXTURE_IMAGE_UNITS", 35660)?;
    constant(scope, object, "MAX_TEXTURE_IMAGE_UNITS", 34930)?;
    constant(scope, object, "MAX_FRAGMENT_UNIFORM_VECTORS", 36349)?;
    constant(scope, object, "SHADER_TYPE", 35663)?;
    constant(scope, object, "DELETE_STATUS", 35712)?;
    constant(scope, object, "LINK_STATUS", 35714)?;
    constant(scope, object, "VALIDATE_STATUS", 35715)?;
    constant(scope, object, "ATTACHED_SHADERS", 35717)?;
    constant(scope, object, "ACTIVE_UNIFORMS", 35718)?;
    constant(scope, object, "ACTIVE_ATTRIBUTES", 35721)?;
    constant(scope, object, "SHADING_LANGUAGE_VERSION", 35724)?;
    constant(scope, object, "CURRENT_PROGRAM", 35725)?;
    constant(scope, object, "NEVER", 512)?;
    constant(scope, object, "LESS", 513)?;
    constant(scope, object, "EQUAL", 514)?;
    constant(scope, object, "LEQUAL", 515)?;
    constant(scope, object, "GREATER", 516)?;
    constant(scope, object, "NOTEQUAL", 517)?;
    constant(scope, object, "GEQUAL", 518)?;
    constant(scope, object, "ALWAYS", 519)?;
    constant(scope, object, "KEEP", 7680)?;
    constant(scope, object, "REPLACE", 7681)?;
    constant(scope, object, "INCR", 7682)?;
    constant(scope, object, "DECR", 7683)?;
    constant(scope, object, "INVERT", 5386)?;
    constant(scope, object, "INCR_WRAP", 34055)?;
    constant(scope, object, "DECR_WRAP", 34056)?;
    constant(scope, object, "VENDOR", 7936)?;
    constant(scope, object, "RENDERER", 7937)?;
    constant(scope, object, "VERSION", 7938)?;
    constant(scope, object, "NEAREST", 9728)?;
    constant(scope, object, "LINEAR", 9729)?;
    constant(scope, object, "NEAREST_MIPMAP_NEAREST", 9984)?;
    constant(scope, object, "LINEAR_MIPMAP_NEAREST", 9985)?;
    constant(scope, object, "NEAREST_MIPMAP_LINEAR", 9986)?;
    constant(scope, object, "LINEAR_MIPMAP_LINEAR", 9987)?;
    constant(scope, object, "TEXTURE_MAG_FILTER", 10240)?;
    constant(scope, object, "TEXTURE_MIN_FILTER", 10241)?;
    constant(scope, object, "TEXTURE_WRAP_S", 10242)?;
    constant(scope, object, "TEXTURE_WRAP_T", 10243)?;
    constant(scope, object, "TEXTURE", 5890)?;
    constant(scope, object, "TEXTURE_CUBE_MAP", 34067)?;
    constant(scope, object, "TEXTURE_BINDING_CUBE_MAP", 34068)?;
    constant(scope, object, "TEXTURE_CUBE_MAP_POSITIVE_X", 34069)?;
    constant(scope, object, "TEXTURE_CUBE_MAP_NEGATIVE_X", 34070)?;
    constant(scope, object, "TEXTURE_CUBE_MAP_POSITIVE_Y", 34071)?;
    constant(scope, object, "TEXTURE_CUBE_MAP_NEGATIVE_Y", 34072)?;
    constant(scope, object, "TEXTURE_CUBE_MAP_POSITIVE_Z", 34073)?;
    constant(scope, object, "TEXTURE_CUBE_MAP_NEGATIVE_Z", 34074)?;
    constant(scope, object, "MAX_CUBE_MAP_TEXTURE_SIZE", 34076)
}

fn define_webgl_constants_194_through_300(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    constant(scope, object, "TEXTURE0", 33984)?;
    constant(scope, object, "TEXTURE1", 33985)?;
    constant(scope, object, "TEXTURE2", 33986)?;
    constant(scope, object, "TEXTURE3", 33987)?;
    constant(scope, object, "TEXTURE4", 33988)?;
    constant(scope, object, "TEXTURE5", 33989)?;
    constant(scope, object, "TEXTURE6", 33990)?;
    constant(scope, object, "TEXTURE7", 33991)?;
    constant(scope, object, "TEXTURE8", 33992)?;
    constant(scope, object, "TEXTURE9", 33993)?;
    constant(scope, object, "TEXTURE10", 33994)?;
    constant(scope, object, "TEXTURE11", 33995)?;
    constant(scope, object, "TEXTURE12", 33996)?;
    constant(scope, object, "TEXTURE13", 33997)?;
    constant(scope, object, "TEXTURE14", 33998)?;
    constant(scope, object, "TEXTURE15", 33999)?;
    constant(scope, object, "TEXTURE16", 34000)?;
    constant(scope, object, "TEXTURE17", 34001)?;
    constant(scope, object, "TEXTURE18", 34002)?;
    constant(scope, object, "TEXTURE19", 34003)?;
    constant(scope, object, "TEXTURE20", 34004)?;
    constant(scope, object, "TEXTURE21", 34005)?;
    constant(scope, object, "TEXTURE22", 34006)?;
    constant(scope, object, "TEXTURE23", 34007)?;
    constant(scope, object, "TEXTURE24", 34008)?;
    constant(scope, object, "TEXTURE25", 34009)?;
    constant(scope, object, "TEXTURE26", 34010)?;
    constant(scope, object, "TEXTURE27", 34011)?;
    constant(scope, object, "TEXTURE28", 34012)?;
    constant(scope, object, "TEXTURE29", 34013)?;
    constant(scope, object, "TEXTURE30", 34014)?;
    constant(scope, object, "TEXTURE31", 34015)?;
    constant(scope, object, "ACTIVE_TEXTURE", 34016)?;
    constant(scope, object, "REPEAT", 10497)?;
    constant(scope, object, "CLAMP_TO_EDGE", 33071)?;
    constant(scope, object, "MIRRORED_REPEAT", 33648)?;
    constant(scope, object, "FLOAT_VEC2", 35664)?;
    constant(scope, object, "FLOAT_VEC3", 35665)?;
    constant(scope, object, "FLOAT_VEC4", 35666)?;
    constant(scope, object, "INT_VEC2", 35667)?;
    constant(scope, object, "INT_VEC3", 35668)?;
    constant(scope, object, "INT_VEC4", 35669)?;
    constant(scope, object, "BOOL", 35670)?;
    constant(scope, object, "BOOL_VEC2", 35671)?;
    constant(scope, object, "BOOL_VEC3", 35672)?;
    constant(scope, object, "BOOL_VEC4", 35673)?;
    constant(scope, object, "FLOAT_MAT2", 35674)?;
    constant(scope, object, "FLOAT_MAT3", 35675)?;
    constant(scope, object, "FLOAT_MAT4", 35676)?;
    constant(scope, object, "SAMPLER_2D", 35678)?;
    constant(scope, object, "SAMPLER_CUBE", 35680)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_ENABLED", 34338)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_SIZE", 34339)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_STRIDE", 34340)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_TYPE", 34341)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_NORMALIZED", 34922)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_POINTER", 34373)?;
    constant(scope, object, "VERTEX_ATTRIB_ARRAY_BUFFER_BINDING", 34975)?;
    constant(scope, object, "IMPLEMENTATION_COLOR_READ_TYPE", 35738)?;
    constant(scope, object, "IMPLEMENTATION_COLOR_READ_FORMAT", 35739)?;
    constant(scope, object, "COMPILE_STATUS", 35713)?;
    constant(scope, object, "LOW_FLOAT", 36336)?;
    constant(scope, object, "MEDIUM_FLOAT", 36337)?;
    constant(scope, object, "HIGH_FLOAT", 36338)?;
    constant(scope, object, "LOW_INT", 36339)?;
    constant(scope, object, "MEDIUM_INT", 36340)?;
    constant(scope, object, "HIGH_INT", 36341)?;
    constant(scope, object, "FRAMEBUFFER", 36160)?;
    constant(scope, object, "RENDERBUFFER", 36161)?;
    constant(scope, object, "RGBA4", 32854)?;
    constant(scope, object, "RGB5_A1", 32855)?;
    constant(scope, object, "RGB565", 36194)?;
    constant(scope, object, "DEPTH_COMPONENT16", 33189)?;
    constant(scope, object, "STENCIL_INDEX8", 36168)?;
    constant(scope, object, "DEPTH_STENCIL", 34041)?;
    constant(scope, object, "RENDERBUFFER_WIDTH", 36162)?;
    constant(scope, object, "RENDERBUFFER_HEIGHT", 36163)?;
    constant(scope, object, "RENDERBUFFER_INTERNAL_FORMAT", 36164)?;
    constant(scope, object, "RENDERBUFFER_RED_SIZE", 36176)?;
    constant(scope, object, "RENDERBUFFER_GREEN_SIZE", 36177)?;
    constant(scope, object, "RENDERBUFFER_BLUE_SIZE", 36178)?;
    constant(scope, object, "RENDERBUFFER_ALPHA_SIZE", 36179)?;
    constant(scope, object, "RENDERBUFFER_DEPTH_SIZE", 36180)?;
    constant(scope, object, "RENDERBUFFER_STENCIL_SIZE", 36181)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE", 36048)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_OBJECT_NAME", 36049)?;
    constant(scope, object, "FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL", 36050)?;
    constant(
        scope,
        object,
        "FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE",
        36051,
    )?;
    constant(scope, object, "COLOR_ATTACHMENT0", 36064)?;
    constant(scope, object, "DEPTH_ATTACHMENT", 36096)?;
    constant(scope, object, "STENCIL_ATTACHMENT", 36128)?;
    constant(scope, object, "DEPTH_STENCIL_ATTACHMENT", 33306)?;
    constant(scope, object, "NONE", 0)?;
    constant(scope, object, "FRAMEBUFFER_COMPLETE", 36053)?;
    constant(scope, object, "FRAMEBUFFER_INCOMPLETE_ATTACHMENT", 36054)?;
    constant(
        scope,
        object,
        "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT",
        36055,
    )?;
    constant(scope, object, "FRAMEBUFFER_INCOMPLETE_DIMENSIONS", 36057)?;
    constant(scope, object, "FRAMEBUFFER_UNSUPPORTED", 36061)?;
    constant(scope, object, "FRAMEBUFFER_BINDING", 36006)?;
    constant(scope, object, "RENDERBUFFER_BINDING", 36007)?;
    constant(scope, object, "MAX_RENDERBUFFER_SIZE", 34024)?;
    constant(scope, object, "INVALID_FRAMEBUFFER_OPERATION", 1286)?;
    constant(scope, object, "UNPACK_FLIP_Y_WEBGL", 37440)?;
    constant(scope, object, "UNPACK_PREMULTIPLY_ALPHA_WEBGL", 37441)?;
    constant(scope, object, "CONTEXT_LOST_WEBGL", 37442)?;
    constant(scope, object, "UNPACK_COLORSPACE_CONVERSION_WEBGL", 37443)?;
    constant(scope, object, "BROWSER_DEFAULT_WEBGL", 37444)
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
        return Err("cannot create WebGLRenderingContext".to_owned());
    }
    let canvas = canvas.map(|canvas| v8::Global::new(scope, canvas));
    scope
        .get_slot_mut::<WebGlRenderingContextStore>()
        .ok_or_else(|| "WebGLRenderingContext state was not prepared".to_owned())?
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
                buffers: HashMap::new(),
                shaders: HashMap::new(),
                programs: HashMap::new(),
                framebuffers: HashMap::new(),
                renderbuffers: HashMap::new(),
                textures: HashMap::new(),
                uniform_locations: HashMap::new(),
                bound_array_buffer: None,
                bound_element_array_buffer: None,
                bound_framebuffer: None,
                bound_renderbuffer: None,
                bound_texture_2d: None,
                bound_texture_cube_map: None,
                current_program: None,
                active_texture: 0x84C0,
                enabled_capabilities: HashSet::new(),
                clear_color: [0.0, 0.0, 0.0, 0.0],
                clear_depth: 1.0,
                clear_stencil: 0,
                color_mask: [true, true, true, true],
                depth_function: 0x0201,
                depth_write_mask: true,
                depth_range: [0.0, 1.0],
                cull_face_mode: 0x0405,
                front_face: 0x0901,
                line_width: 1.0,
                blend_color: [0.0, 0.0, 0.0, 0.0],
                blend_equation: [0x8006, 0x8006],
                blend_function: [1, 0, 1, 0],
                pixel_store: HashMap::new(),
                polygon_offset: [0.0, 0.0],
                sample_coverage: [1.0, 0.0],
                scissor: [0, 0, width as i32, height as i32],
                vertex_attrib_enabled: HashSet::new(),
                vertex_attrib_values: HashMap::new(),
                vertex_attrib_pointers: HashMap::new(),
                extensions: HashMap::new(),
                hints: HashMap::new(),
                stencil_front: [0x0207, 0, u32::MAX, u32::MAX, 0x1E00, 0x1E00, 0x1E00],
                stencil_back: [0x0207, 0, u32::MAX, u32::MAX, 0x1E00, 0x1E00, 0x1E00],
                viewport: [0, 0, width as i32, height as i32],
                draw_calls: 0,
            },
        );
    Ok(context)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ContextRecord> {
    scope
        .get_slot::<WebGlRenderingContextStore>()?
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
        .get_slot_mut::<WebGlRenderingContextStore>()
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
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.width).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_drawing_buffer_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.height).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn return_string(scope: &v8::PinScope<'_, '_>, result: &mut v8::ReturnValue<'_>, value: &str) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into())
    }
}
fn get_drawing_buffer_color_space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &mut r, &v.drawing_buffer_color_space)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_drawing_buffer_color_space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |r| r.drawing_buffer_color_space = v);
}
fn get_unpack_color_space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &mut r, &v.unpack_color_space)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_unpack_color_space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |r| r.unpack_color_space = v);
}

fn active_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |r| {
        if !(0x84C0..=0x84DF).contains(&value) {
            set_error(r, INVALID_ENUM)
        } else {
            r.active_texture = value;
        }
    });
}

fn create_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Ok(object) = super::webgl_buffer::create(s) {
        let id = object.get_identity_hash().get();
        update(s, a.this(), |v| {
            v.buffers.insert(
                id,
                BufferRecord {
                    size: 0,
                    usage: 0x88E4,
                    deleted: false,
                },
            );
        });
        r.set(object.into())
    }
}
fn create_framebuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(object) = super::webgl_framebuffer::create(s) {
        let id = object.get_identity_hash().get();
        if update(s, a.this(), |v| {
            v.framebuffers.insert(id, FramebufferRecord::default());
        }) {
            r.set(object.into())
        }
    }
}
fn create_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(object) = super::webgl_program::create(s) {
        let id = object.get_identity_hash().get();
        if update(s, a.this(), |v| {
            v.programs.insert(
                id,
                ProgramRecord {
                    attached_shaders: Vec::new(),
                    bound_attributes: HashMap::new(),
                    active_attributes: Vec::new(),
                    active_uniforms: Vec::new(),
                    linked: false,
                    validated: false,
                    deleted: false,
                    info_log: String::new(),
                },
            );
        }) {
            r.set(object.into())
        }
    }
}
fn create_renderbuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(object) = super::webgl_renderbuffer::create(s) {
        let id = object.get_identity_hash().get();
        if update(s, a.this(), |v| {
            v.renderbuffers.insert(id, RenderbufferRecord::default());
        }) {
            r.set(object.into())
        }
    }
}
fn create_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(object) = super::webgl_texture::create(s) {
        let id = object.get_identity_hash().get();
        if update(s, a.this(), |v| {
            v.textures.insert(id, TextureRecord::default());
        }) {
            r.set(object.into())
        }
    }
}
fn create_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let kind = a.get(0).uint32_value(s).unwrap_or(0);
    if kind != VERTEX_SHADER && kind != FRAGMENT_SHADER {
        update(s, a.this(), |v| set_error(v, INVALID_ENUM));
        r.set(v8::null(s).into());
        return;
    }
    if let Ok(object) = super::webgl_shader::create(s) {
        let id = object.get_identity_hash().get();
        let stored_object = v8::Global::new(s, object);
        if update(s, a.this(), |v| {
            v.shaders.insert(
                id,
                ShaderRecord {
                    object: stored_object,
                    kind,
                    source: String::new(),
                    compiled: false,
                    deleted: false,
                    info_log: String::new(),
                },
            );
        }) {
            r.set(object.into())
        }
    }
}

fn object_id(value: v8::Local<'_, v8::Value>) -> Option<i32> {
    v8::Local::<v8::Object>::try_from(value)
        .ok()
        .map(|object| object.get_identity_hash().get())
}

fn delete_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            if let Some(x) = v.buffers.get_mut(&id) {
                x.deleted = true
            }
        });
    }
}
fn delete_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            if let Some(x) = v.programs.get_mut(&id) {
                x.deleted = true
            }
        });
    }
}
fn delete_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            if let Some(x) = v.shaders.get_mut(&id) {
                x.deleted = true
            }
        });
    }
}
fn delete_framebuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            v.framebuffers.remove(&id);
        });
    }
}
fn delete_renderbuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            v.renderbuffers.remove(&id);
        });
    }
}
fn delete_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            v.textures.remove(&id);
        });
    }
}

fn shader_source(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = object_id(a.get(0));
    let source = crate::webidl::value_to_string(s, a.get(1));
    if let Some(id) = id {
        update(s, a.this(), |v| {
            if let Some(x) = v.shaders.get_mut(&id) {
                x.source = source;
                x.compiled = false
            }
        });
    }
}
fn compile_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            if let Some(x) = v.shaders.get_mut(&id) {
                x.compiled = !x.source.trim().is_empty();
                x.info_log = if x.compiled {
                    String::new()
                } else {
                    "shader source is empty".to_owned()
                }
            }
        });
    }
}
fn attach_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let p = object_id(a.get(0));
    let sh = object_id(a.get(1));
    if let (Some(p), Some(sh)) = (p, sh) {
        update(s, a.this(), |v| {
            if v.shaders.contains_key(&sh) {
                if let Some(x) = v.programs.get_mut(&p) {
                    if !x.attached_shaders.contains(&sh) {
                        x.attached_shaders.push(sh)
                    }
                }
            }
        });
    }
}
fn detach_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let p = object_id(a.get(0));
    let sh = object_id(a.get(1));
    if let (Some(p), Some(sh)) = (p, sh) {
        update(s, a.this(), |v| {
            if let Some(x) = v.programs.get_mut(&p) {
                x.attached_shaders.retain(|id| *id != sh)
            }
        });
    }
}
fn link_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            let attached = v
                .programs
                .get(&id)
                .map(|program| program.attached_shaders.clone())
                .unwrap_or_default();
            let compiled = !attached.is_empty()
                && attached
                    .iter()
                    .all(|shader| v.shaders.get(shader).is_some_and(|shader| shader.compiled));
            let mut attributes = Vec::new();
            let mut uniforms = Vec::new();
            if compiled {
                for shader_id in &attached {
                    if let Some(shader) = v.shaders.get(shader_id) {
                        if shader.kind == VERTEX_SHADER {
                            append_declarations(&shader.source, "attribute", &mut attributes);
                            append_declarations(&shader.source, "in", &mut attributes);
                        }
                        append_declarations(&shader.source, "uniform", &mut uniforms);
                    }
                }
            }
            if let Some(p) = v.programs.get_mut(&id) {
                p.linked = compiled;
                p.active_attributes = attributes;
                p.active_uniforms = uniforms;
                p.info_log = if compiled {
                    String::new()
                } else {
                    "attached shader compilation failed".to_owned()
                }
            }
        });
    }
}
fn validate_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(id) = object_id(a.get(0)) {
        update(s, a.this(), |v| {
            if let Some(p) = v.programs.get_mut(&id) {
                p.validated = p.linked
            }
        });
    }
}
fn use_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.get(0).is_null() {
        update(s, a.this(), |v| v.current_program = None);
        return;
    }
    let object = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    if let Some(object) = object {
        let id = object.get_identity_hash().get();
        let global = v8::Global::new(s, object);
        update(s, a.this(), |v| {
            if v.programs.get(&id).is_some_and(|p| p.linked && !p.deleted) {
                v.current_program = Some(global)
            } else {
                set_error(v, INVALID_OPERATION)
            }
        });
    }
}

fn get_shader_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let id = object_id(a.get(0));
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    let value = id.and_then(|id| record(s, a.this())?.shaders.get(&id).cloned());
    let Some(v) = value else {
        r.set(v8::null(s).into());
        return;
    };
    match parameter {
        SHADER_TYPE => r.set(v8::Integer::new_from_unsigned(s, v.kind).into()),
        DELETE_STATUS => r.set(v8::Boolean::new(s, v.deleted).into()),
        COMPILE_STATUS => r.set(v8::Boolean::new(s, v.compiled).into()),
        _ => r.set(v8::null(s).into()),
    }
}
fn get_shader_source(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let value = object_id(a.get(0)).and_then(|id| {
        record(s, a.this())?
            .shaders
            .get(&id)
            .map(|v| v.source.clone())
    });
    if let Some(v) = value {
        return_string(s, &mut r, &v)
    } else {
        r.set(v8::null(s).into())
    }
}
fn get_shader_info_log(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let value = object_id(a.get(0)).and_then(|id| {
        record(s, a.this())?
            .shaders
            .get(&id)
            .map(|v| v.info_log.clone())
    });
    if let Some(v) = value {
        return_string(s, &mut r, &v)
    } else {
        r.set(v8::null(s).into())
    }
}
fn get_program_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let id = object_id(a.get(0));
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    let value = id.and_then(|id| record(s, a.this())?.programs.get(&id).cloned());
    let Some(v) = value else {
        r.set(v8::null(s).into());
        return;
    };
    match parameter {
        DELETE_STATUS => r.set(v8::Boolean::new(s, v.deleted).into()),
        LINK_STATUS => r.set(v8::Boolean::new(s, v.linked).into()),
        VALIDATE_STATUS => r.set(v8::Boolean::new(s, v.validated).into()),
        ATTACHED_SHADERS => r.set(v8::Integer::new(s, v.attached_shaders.len() as i32).into()),
        ACTIVE_UNIFORMS => r.set(v8::Integer::new(s, v.active_uniforms.len() as i32).into()),
        ACTIVE_ATTRIBUTES => r.set(v8::Integer::new(s, v.active_attributes.len() as i32).into()),
        _ => r.set(v8::null(s).into()),
    }
}
fn get_program_info_log(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let value = object_id(a.get(0)).and_then(|id| {
        record(s, a.this())?
            .programs
            .get(&id)
            .map(|v| v.info_log.clone())
    });
    if let Some(v) = value {
        return_string(s, &mut r, &v)
    } else {
        r.set(v8::null(s).into())
    }
}
fn get_attached_shaders(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let shaders = object_id(a.get(0))
        .and_then(|id| {
            let context = record(s, a.this())?;
            let program = context.programs.get(&id)?;
            Some(
                program
                    .attached_shaders
                    .iter()
                    .filter_map(|shader_id| context.shaders.get(shader_id))
                    .filter(|shader| !shader.deleted)
                    .map(|shader| shader.object.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_default();
    let array = v8::Array::new(s, shaders.len() as i32);
    for (index, shader) in shaders.iter().enumerate() {
        let value = v8::Local::new(s, shader);
        let _ = array.set_index(s, index as u32, value.into());
    }
    r.set(array.into())
}

fn bind_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let value = if a.get(1).is_null() {
        None
    } else {
        let Some(object) = v8::Local::<v8::Object>::try_from(a.get(1)).ok() else {
            update(s, a.this(), |context| set_error(context, INVALID_OPERATION));
            return;
        };
        let id = object.get_identity_hash().get();
        let valid = record(s, a.this())
            .and_then(|context| context.buffers.get(&id).cloned())
            .is_some_and(|buffer| !buffer.deleted);
        if !valid {
            update(s, a.this(), |context| set_error(context, INVALID_OPERATION));
            return;
        }
        Some(v8::Global::new(s, object))
    };
    update(s, a.this(), |context| match target {
        ARRAY_BUFFER => context.bound_array_buffer = value,
        ELEMENT_ARRAY_BUFFER => context.bound_element_array_buffer = value,
        _ => set_error(context, INVALID_ENUM),
    });
}
fn buffer_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let usage = a.get(2).uint32_value(s).unwrap_or(0);
    let size = if a.get(1).is_number() {
        a.get(1).uint32_value(s).unwrap_or(0)
    } else {
        v8::Local::<v8::Object>::try_from(a.get(1))
            .ok()
            .and_then(|object| {
                v8::String::new(s, "byteLength").and_then(|key| object.get(s, key.into()))
            })
            .and_then(|value| value.uint32_value(s))
            .unwrap_or(0)
    };
    let bound_id = bound_buffer_id(s, a.this(), target);
    update(s, a.this(), |context| {
        if target != ARRAY_BUFFER && target != ELEMENT_ARRAY_BUFFER {
            set_error(context, INVALID_ENUM);
            return;
        }
        let Some(id) = bound_id else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        let Some(buffer) = context.buffers.get_mut(&id) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        if buffer.deleted {
            set_error(context, INVALID_OPERATION);
            return;
        }
        buffer.size = size;
        buffer.usage = usage;
    });
}
fn buffer_sub_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let offset = a.get(1).uint32_value(s).unwrap_or(0);
    let data_size = v8::Local::<v8::Object>::try_from(a.get(2))
        .ok()
        .and_then(|object| {
            v8::String::new(s, "byteLength").and_then(|key| object.get(s, key.into()))
        })
        .and_then(|value| value.uint32_value(s))
        .unwrap_or(0);
    let bound_id = bound_buffer_id(s, a.this(), target);
    update(s, a.this(), |context| {
        if target != ARRAY_BUFFER && target != ELEMENT_ARRAY_BUFFER {
            set_error(context, INVALID_ENUM);
            return;
        }
        let Some(id) = bound_id else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        let Some(buffer) = context.buffers.get(&id) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        if buffer.deleted || offset.saturating_add(data_size) > buffer.size {
            set_error(context, INVALID_VALUE);
        }
    });
}

fn get_buffer_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    if target != ARRAY_BUFFER && target != ELEMENT_ARRAY_BUFFER {
        update(s, a.this(), |context| set_error(context, INVALID_ENUM));
        r.set(v8::null(s).into());
        return;
    }
    let bound_id = bound_buffer_id(s, a.this(), target);
    let context = record(s, a.this());
    let buffer = bound_id.and_then(|id| context.as_ref()?.buffers.get(&id));
    match (parameter, buffer) {
        (BUFFER_SIZE, Some(buffer)) => r.set(v8::Integer::new_from_unsigned(s, buffer.size).into()),
        (BUFFER_USAGE, Some(buffer)) => {
            r.set(v8::Integer::new_from_unsigned(s, buffer.usage).into())
        }
        _ => r.set(v8::null(s).into()),
    }
}

fn bound_buffer_id(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    target: u32,
) -> Option<i32> {
    let context = record(scope, context)?;
    let buffer = match target {
        ARRAY_BUFFER => context.bound_array_buffer?,
        ELEMENT_ARRAY_BUFFER => context.bound_element_array_buffer?,
        _ => return None,
    };
    Some(v8::Local::new(scope, &buffer).get_identity_hash().get())
}

fn get_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let error = record(s, a.this()).map(|v| v.error).unwrap_or(NO_ERROR);
    update(s, a.this(), |v| v.error = NO_ERROR);
    r.set(v8::Integer::new_from_unsigned(s, error).into())
}
fn get_context_attributes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let configured = crate::fingerprint::edge(s).rendering.webgl.clone();
    let o = v8::Object::new(s);
    define_data(
        s,
        o,
        "alpha",
        v8::Boolean::new(s, configured.context_alpha).into(),
    );
    define_data(
        s,
        o,
        "antialias",
        v8::Boolean::new(s, configured.context_antialias).into(),
    );
    define_data(
        s,
        o,
        "depth",
        v8::Boolean::new(s, configured.context_depth).into(),
    );
    define_data(
        s,
        o,
        "desynchronized",
        v8::Boolean::new(s, configured.context_desynchronized).into(),
    );
    define_data(
        s,
        o,
        "failIfMajorPerformanceCaveat",
        v8::Boolean::new(s, configured.context_fail_if_major_performance_caveat).into(),
    );
    if let Some(power_preference) = v8::String::new(s, &configured.context_power_preference) {
        define_data(s, o, "powerPreference", power_preference.into());
    }
    define_data(
        s,
        o,
        "premultipliedAlpha",
        v8::Boolean::new(s, configured.context_premultiplied_alpha).into(),
    );
    define_data(
        s,
        o,
        "preserveDrawingBuffer",
        v8::Boolean::new(s, configured.context_preserve_drawing_buffer).into(),
    );
    define_data(
        s,
        o,
        "stencil",
        v8::Boolean::new(s, configured.context_stencil).into(),
    );
    define_data(
        s,
        o,
        "xrCompatible",
        v8::Boolean::new(s, configured.context_xr_compatible).into(),
    );
    r.set(o.into())
}
fn get_supported_extensions(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        let extensions = crate::fingerprint::edge(s)
            .rendering
            .webgl
            .webgl1_extensions
            .clone();
        r.set(string_array(s, &extensions).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_extension(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(context) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let requested = crate::webidl::value_to_string(s, a.get(0));
    let supported = crate::fingerprint::edge(s)
        .rendering
        .webgl
        .webgl1_extensions
        .iter()
        .find(|name| name.eq_ignore_ascii_case(&requested))
        .cloned();
    let Some(name) = supported else {
        r.set(v8::null(s).into());
        return;
    };
    if let Some(existing) = context.extensions.get(&name) {
        r.set(v8::Local::new(s, existing).into());
        return;
    }
    let extension = extension_object(s, &name);
    let stored = v8::Global::new(s, extension);
    update(s, a.this(), |context| {
        context.extensions.insert(name, stored);
    });
    r.set(extension.into());
}
fn get_shader_precision_format(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let configured = crate::fingerprint::edge(s).rendering.webgl.clone();
    match super::webgl_shader_precision_format::create(
        s,
        configured.shader_precision_range_min,
        configured.shader_precision_range_max,
        configured.shader_precision_bits,
    ) {
        Ok(v) => r.set(v.into()),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn get_uniform_location(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let program = object_id(a.get(0));
    let name = crate::webidl::value_to_string(s, a.get(1));
    let existing = program.and_then(|program| {
        record(s, a.this())?
            .uniform_locations
            .values()
            .find(|location| location.program == program && location.name == name)
            .map(|location| location.object.clone())
    });
    if let Some(existing) = existing {
        r.set(v8::Local::new(s, &existing).into());
        return;
    }
    let valid = program
        .and_then(|id| {
            let context = record(s, a.this())?;
            let program = context.programs.get(&id)?;
            Some(program.linked && program.active_uniforms.iter().any(|item| item == &name))
        })
        .unwrap_or(false);
    if !valid {
        r.set(v8::null(s).into());
        return;
    }
    match super::webgl_uniform_location::create(s) {
        Ok(location) => {
            let id = location.get_identity_hash().get();
            let object = v8::Global::new(s, location);
            let program = program.unwrap_or_default();
            update(s, a.this(), |context| {
                context.uniform_locations.insert(
                    id,
                    UniformRecord {
                        object,
                        program,
                        name,
                        values: vec![0.0],
                    },
                );
            });
            r.set(location.into())
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn get_active_attrib(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let program = object_id(a.get(0));
    let index = a.get(1).uint32_value(s).unwrap_or(0) as usize;
    let name = program.and_then(|id| {
        record(s, a.this())?
            .programs
            .get(&id)?
            .active_attributes
            .get(index)
            .cloned()
    });
    match name {
        Some(name) => match super::webgl_active_info::create(s, 1, 0x1406, name) {
            Ok(info) => r.set(info.into()),
            Err(error) => crate::webidl::throw_type_error(s, &error),
        },
        None => r.set(v8::null(s).into()),
    }
}
fn get_active_uniform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let program = object_id(a.get(0));
    let index = a.get(1).uint32_value(s).unwrap_or(0) as usize;
    let name = program.and_then(|id| {
        record(s, a.this())?
            .programs
            .get(&id)?
            .active_uniforms
            .get(index)
            .cloned()
    });
    match name {
        Some(name) => match super::webgl_active_info::create(s, 1, 0x1406, name) {
            Ok(info) => r.set(info.into()),
            Err(error) => crate::webidl::throw_type_error(s, &error),
        },
        None => r.set(v8::null(s).into()),
    }
}
fn get_attrib_location(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let program = object_id(a.get(0));
    let name = crate::webidl::value_to_string(s, a.get(1));
    let location = program
        .and_then(|id| {
            let context = record(s, a.this())?;
            let program = context.programs.get(&id)?;
            program.bound_attributes.get(&name).copied().or_else(|| {
                program
                    .active_attributes
                    .iter()
                    .position(|item| item == &name)
                    .map(|index| index as u32)
            })
        })
        .map(|value| value as i32)
        .unwrap_or(-1);
    r.set(v8::Integer::new(s, location).into())
}

fn get_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let parameter = a.get(0).uint32_value(s).unwrap_or(0);
    let Some(context) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    match parameter {
        34964 => return_global_or_null(s, &mut r, context.bound_array_buffer.as_ref()),
        34965 => return_global_or_null(s, &mut r, context.bound_element_array_buffer.as_ref()),
        32873 => return_global_or_null(s, &mut r, context.bound_texture_2d.as_ref()),
        34068 => return_global_or_null(s, &mut r, context.bound_texture_cube_map.as_ref()),
        36006 => return_global_or_null(s, &mut r, context.bound_framebuffer.as_ref()),
        36007 => return_global_or_null(s, &mut r, context.bound_renderbuffer.as_ref()),
        CURRENT_PROGRAM => return_global_or_null(s, &mut r, context.current_program.as_ref()),
        34016 => r.set(v8::Integer::new_from_unsigned(s, context.active_texture).into()),
        2849 => r.set(v8::Number::new(s, context.line_width).into()),
        2885 => r.set(v8::Integer::new_from_unsigned(s, context.cull_face_mode).into()),
        2886 => r.set(v8::Integer::new_from_unsigned(s, context.front_face).into()),
        2928 => return_number_array(s, &mut r, &context.depth_range),
        2930 => r.set(v8::Boolean::new(s, context.depth_write_mask).into()),
        2931 => r.set(v8::Number::new(s, context.clear_depth).into()),
        2932 => r.set(v8::Integer::new_from_unsigned(s, context.depth_function).into()),
        2961 => r.set(v8::Integer::new(s, context.clear_stencil).into()),
        2962 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[0]).into()),
        2963 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[2]).into()),
        2964 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[4]).into()),
        2965 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[5]).into()),
        2966 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[6]).into()),
        2967 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[1]).into()),
        2968 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_front[3]).into()),
        34816 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[0]).into()),
        34817 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[4]).into()),
        34818 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[5]).into()),
        34819 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[6]).into()),
        36003 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[1]).into()),
        36004 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[2]).into()),
        36005 => r.set(v8::Integer::new_from_unsigned(s, context.stencil_back[3]).into()),
        2978 => return_integer_array(s, &mut r, &context.viewport),
        3088 => return_integer_array(s, &mut r, &context.scissor),
        3106 => return_number_array(s, &mut r, &context.clear_color),
        3107 => return_boolean_array(s, &mut r, &context.color_mask),
        32773 => return_number_array(s, &mut r, &context.blend_color),
        32777 => r.set(v8::Integer::new_from_unsigned(s, context.blend_equation[0]).into()),
        34877 => r.set(v8::Integer::new_from_unsigned(s, context.blend_equation[1]).into()),
        32968 => r.set(v8::Integer::new_from_unsigned(s, context.blend_function[1]).into()),
        32969 => r.set(v8::Integer::new_from_unsigned(s, context.blend_function[0]).into()),
        32970 => r.set(v8::Integer::new_from_unsigned(s, context.blend_function[3]).into()),
        32971 => r.set(v8::Integer::new_from_unsigned(s, context.blend_function[2]).into()),
        32938 => r.set(v8::Number::new(s, context.sample_coverage[0]).into()),
        32939 => r.set(v8::Boolean::new(s, context.sample_coverage[1] != 0.0).into()),
        10752 => r.set(v8::Number::new(s, context.polygon_offset[1]).into()),
        32824 => r.set(v8::Number::new(s, context.polygon_offset[0]).into()),
        3317 | 3333 | 37440 | 37441 | 37443 => {
            let value = context
                .pixel_store
                .get(&parameter)
                .copied()
                .unwrap_or_else(|| {
                    if parameter == 3317 || parameter == 3333 {
                        4
                    } else if parameter == 37443 {
                        37444
                    } else {
                        0
                    }
                });
            r.set(v8::Integer::new(s, value).into());
        }
        7936 => {
            let value = crate::fingerprint::edge(s).rendering.webgl.vendor.clone();
            return_string(s, &mut r, &value)
        }
        7937 => {
            let value = crate::fingerprint::edge(s).rendering.webgl.renderer.clone();
            return_string(s, &mut r, &value)
        }
        37445 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .unmasked_vendor
                .clone();
            return_string(s, &mut r, &value)
        }
        37446 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .unmasked_renderer
                .clone();
            return_string(s, &mut r, &value)
        }
        7938 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .webgl1_version
                .clone();
            return_string(s, &mut r, &value)
        }
        35724 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .webgl1_shading_language_version
                .clone();
            return_string(s, &mut r, &value)
        }
        3379 => {
            let value = crate::fingerprint::edge(s).rendering.webgl.max_texture_size;
            r.set(v8::Integer::new(s, value).into())
        }
        34076 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_cube_map_texture_size;
            r.set(v8::Integer::new(s, value).into())
        }
        34024 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_renderbuffer_size;
            r.set(v8::Integer::new(s, value).into())
        }
        34047 => {
            let value = crate::fingerprint::edge(s).rendering.webgl.max_anisotropy;
            r.set(v8::Number::new(s, value).into())
        }
        34921 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_vertex_attribs;
            r.set(v8::Integer::new(s, value).into())
        }
        36347 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_vertex_uniform_vectors;
            r.set(v8::Integer::new(s, value).into())
        }
        36348 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_varying_vectors;
            r.set(v8::Integer::new(s, value).into())
        }
        36349 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_fragment_uniform_vectors;
            r.set(v8::Integer::new(s, value).into())
        }
        35660 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_vertex_texture_image_units;
            r.set(v8::Integer::new(s, value).into())
        }
        34930 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_texture_image_units;
            r.set(v8::Integer::new(s, value).into())
        }
        35661 => {
            let value = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .max_combined_texture_image_units;
            r.set(v8::Integer::new(s, value).into())
        }
        3408 => {
            let value = crate::fingerprint::edge(s).rendering.webgl.subpixel_bits;
            r.set(v8::Integer::new(s, value).into())
        }
        33901 => {
            let configured = &crate::fingerprint::edge(s).rendering.webgl;
            return_number_array(
                s,
                &mut r,
                &[
                    configured.aliased_point_size_min,
                    configured.aliased_point_size_max,
                ],
            )
        }
        33902 => {
            let configured = &crate::fingerprint::edge(s).rendering.webgl;
            return_number_array(
                s,
                &mut r,
                &[
                    configured.aliased_line_width_min,
                    configured.aliased_line_width_max,
                ],
            )
        }
        3386 => {
            let configured = &crate::fingerprint::edge(s).rendering.webgl;
            return_integer_array(
                s,
                &mut r,
                &[
                    configured.max_viewport_width,
                    configured.max_viewport_height,
                ],
            )
        }
        34467 => {
            let configured = crate::fingerprint::edge(s)
                .rendering
                .webgl
                .compressed_texture_formats
                .clone();
            let array = v8::Array::new(s, configured.len() as i32);
            for (index, value) in configured.into_iter().enumerate() {
                let value = v8::Integer::new_from_unsigned(s, value);
                let _ = array.set_index(s, index as u32, value.into());
            }
            r.set(array.into())
        }
        _ => {
            update(s, a.this(), |context| set_error(context, INVALID_ENUM));
            r.set(v8::null(s).into())
        }
    }
}

fn string_array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: &[String],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    array
}

fn extension_object<'s>(scope: &mut v8::PinScope<'s, '_>, name: &str) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    if name.eq_ignore_ascii_case("WEBGL_debug_renderer_info") {
        define_data(
            scope,
            object,
            "UNMASKED_VENDOR_WEBGL",
            v8::Integer::new(scope, 37_445).into(),
        );
        define_data(
            scope,
            object,
            "UNMASKED_RENDERER_WEBGL",
            v8::Integer::new(scope, 37_446).into(),
        );
    }
    if name.eq_ignore_ascii_case("EXT_texture_filter_anisotropic") {
        define_data(
            scope,
            object,
            "TEXTURE_MAX_ANISOTROPY_EXT",
            v8::Integer::new(scope, 34_046).into(),
        );
        define_data(
            scope,
            object,
            "MAX_TEXTURE_MAX_ANISOTROPY_EXT",
            v8::Integer::new(scope, 34_047).into(),
        );
    }
    object
}

fn return_global_or_null(
    scope: &v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    value: Option<&v8::Global<v8::Object>>,
) {
    if let Some(value) = value {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn return_number_array(
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

fn return_integer_array(
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

fn return_boolean_array(
    scope: &v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    values: &[bool],
) {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let boolean = v8::Boolean::new(scope, *value);
        let _ = array.set_index(scope, index as u32, boolean.into());
    }
    result.set(array.into());
}
fn is_context_lost(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Boolean::new(s, false).into())
    }
}
fn return_membership(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&ContextRecord, i32) -> bool,
) {
    let id = object_id(a.get(0));
    let value = record(s, a.this())
        .zip(id)
        .is_some_and(|(record, id)| select(&record, id));
    r.set(v8::Boolean::new(s, value).into())
}
fn is_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_membership(s, a, r, |v, id| {
        v.buffers.get(&id).is_some_and(|b| !b.deleted)
    })
}
fn is_framebuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_membership(s, a, r, |v, id| v.framebuffers.contains_key(&id))
}
fn is_program(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_membership(s, a, r, |v, id| {
        v.programs.get(&id).is_some_and(|p| !p.deleted)
    })
}
fn is_renderbuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_membership(s, a, r, |v, id| v.renderbuffers.contains_key(&id))
}
fn is_shader(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_membership(s, a, r, |v, id| {
        v.shaders.get(&id).is_some_and(|p| !p.deleted)
    })
}
fn is_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_membership(s, a, r, |v, id| v.textures.contains_key(&id))
}
fn enable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let cap = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |v| {
        v.enabled_capabilities.insert(cap);
    });
}
fn disable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let cap = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |v| {
        v.enabled_capabilities.remove(&cap);
    });
}
fn is_enabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let cap = a.get(0).uint32_value(s).unwrap_or(0);
    let value = record(s, a.this()).is_some_and(|v| v.enabled_capabilities.contains(&cap));
    r.set(v8::Boolean::new(s, value).into())
}

fn clear_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let c = [
        a.get(0).number_value(s).unwrap_or(0.0),
        a.get(1).number_value(s).unwrap_or(0.0),
        a.get(2).number_value(s).unwrap_or(0.0),
        a.get(3).number_value(s).unwrap_or(0.0),
    ];
    update(s, a.this(), |v| v.clear_color = c);
}
fn clear_depth(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let d = a.get(0).number_value(s).unwrap_or(1.0).clamp(0.0, 1.0);
    update(s, a.this(), |v| v.clear_depth = d);
}
fn clear_stencil(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).int32_value(s).unwrap_or(0);
    update(s, a.this(), |r| r.clear_stencil = v);
}
fn viewport(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = [
        a.get(0).int32_value(s).unwrap_or(0),
        a.get(1).int32_value(s).unwrap_or(0),
        a.get(2).int32_value(s).unwrap_or(0),
        a.get(3).int32_value(s).unwrap_or(0),
    ];
    update(s, a.this(), |r| r.viewport = v);
}
fn clear(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mask = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |r| {
        if mask & !0x4500 != 0 {
            set_error(r, INVALID_VALUE)
        }
    });
}
fn draw_arrays(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let count = a.get(2).int32_value(s).unwrap_or(0);
    update(s, a.this(), |r| {
        if count < 0 {
            set_error(r, INVALID_VALUE)
        } else {
            r.draw_calls += 1
        }
    });
}
fn draw_elements(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let count = a.get(1).int32_value(s).unwrap_or(0);
    update(s, a.this(), |r| {
        if count < 0 {
            set_error(r, INVALID_VALUE)
        } else {
            r.draw_calls += 1
        }
    });
}

fn bind_framebuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    if target != 0x8D40 {
        update(s, a.this(), |context| set_error(context, INVALID_ENUM));
        return;
    }
    let binding = tracked_framebuffer(s, a.this(), a.get(1));
    update(s, a.this(), |context| match binding {
        Some(binding) => context.bound_framebuffer = binding,
        None => set_error(context, INVALID_OPERATION),
    });
}
fn bind_renderbuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    if target != 0x8D41 {
        update(s, a.this(), |context| set_error(context, INVALID_ENUM));
        return;
    }
    let binding = tracked_renderbuffer(s, a.this(), a.get(1));
    update(s, a.this(), |context| match binding {
        Some(binding) => context.bound_renderbuffer = binding,
        None => set_error(context, INVALID_OPERATION),
    });
}
fn bind_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    if target != 0x0DE1 && target != 0x8513 {
        update(s, a.this(), |context| set_error(context, INVALID_ENUM));
        return;
    }
    let binding = tracked_texture(s, a.this(), a.get(1));
    update(s, a.this(), |context| match binding {
        Some(binding) if target == 0x0DE1 => context.bound_texture_2d = binding,
        Some(binding) => context.bound_texture_cube_map = binding,
        None => set_error(context, INVALID_OPERATION),
    });
}
fn check_framebuffer_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Integer::new_from_unsigned(s, FRAMEBUFFER_COMPLETE).into())
    }
}

fn get_drawing_buffer_format(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Integer::new(s, 0x8058).into())
    }
}
fn drawing_buffer_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = a.get(1).uint32_value(s).unwrap_or(0);
    let height = a.get(2).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |r| {
        r.width = width;
        r.height = height;
        r.viewport = [0, 0, width as i32, height as i32]
    });
}

fn tracked_framebuffer(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Option<v8::Global<v8::Object>>> {
    if value.is_null() {
        return Some(None);
    }
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let id = object.get_identity_hash().get();
    record(scope, context)?
        .framebuffers
        .contains_key(&id)
        .then(|| Some(v8::Global::new(scope, object)))
}

fn tracked_renderbuffer(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Option<v8::Global<v8::Object>>> {
    if value.is_null() {
        return Some(None);
    }
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let id = object.get_identity_hash().get();
    record(scope, context)?
        .renderbuffers
        .contains_key(&id)
        .then(|| Some(v8::Global::new(scope, object)))
}

fn tracked_texture(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Option<v8::Global<v8::Object>>> {
    if value.is_null() {
        return Some(None);
    }
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let id = object.get_identity_hash().get();
    record(scope, context)?
        .textures
        .contains_key(&id)
        .then(|| Some(v8::Global::new(scope, object)))
}

fn global_object_id(
    scope: &v8::PinScope<'_, '_>,
    object: Option<&v8::Global<v8::Object>>,
) -> Option<i32> {
    Some(v8::Local::new(scope, object?).get_identity_hash().get())
}

fn bound_texture_id(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    target: u32,
) -> Option<i32> {
    let context = record(scope, context)?;
    if target == 0x0DE1 {
        global_object_id(scope, context.bound_texture_2d.as_ref())
    } else if (0x8513..=0x851A).contains(&target) {
        global_object_id(scope, context.bound_texture_cube_map.as_ref())
    } else {
        None
    }
}

fn bind_attrib_location(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let program = object_id(a.get(0));
    let index = a.get(1).uint32_value(s).unwrap_or(0);
    let name = crate::webidl::value_to_string(s, a.get(2));
    update(s, a.this(), |context| {
        let Some(program) = program.and_then(|id| context.programs.get_mut(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        if program.linked {
            set_error(context, INVALID_OPERATION);
        } else {
            program.bound_attributes.insert(name, index);
        }
    });
}
fn blend_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = [
        a.get(0).number_value(s).unwrap_or(0.0).clamp(0.0, 1.0),
        a.get(1).number_value(s).unwrap_or(0.0).clamp(0.0, 1.0),
        a.get(2).number_value(s).unwrap_or(0.0).clamp(0.0, 1.0),
        a.get(3).number_value(s).unwrap_or(0.0).clamp(0.0, 1.0),
    ];
    update(s, a.this(), |context| context.blend_color = value);
}
fn blend_equation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let equation = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.blend_equation = [equation, equation]
    });
}
fn blend_equation_separate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let rgb = a.get(0).uint32_value(s).unwrap_or(0);
    let alpha = a.get(1).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| context.blend_equation = [rgb, alpha]);
}
fn blend_func(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let source = a.get(0).uint32_value(s).unwrap_or(0);
    let destination = a.get(1).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.blend_function = [source, destination, source, destination]
    });
}
fn blend_func_separate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = [
        a.get(0).uint32_value(s).unwrap_or(0),
        a.get(1).uint32_value(s).unwrap_or(0),
        a.get(2).uint32_value(s).unwrap_or(0),
        a.get(3).uint32_value(s).unwrap_or(0),
    ];
    update(s, a.this(), |context| context.blend_function = value);
}
fn finish(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = record(s, a.this()).or_else(|| {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        None
    });
}
fn flush(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = record(s, a.this()).or_else(|| {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        None
    });
}

fn cull_face(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mode = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| context.cull_face_mode = mode);
}

fn front_face(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mode = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| context.front_face = mode);
}

fn depth_func(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let function = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| context.depth_function = function);
}

fn depth_mask(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let enabled = a.get(0).boolean_value(s);
    update(s, a.this(), |context| context.depth_write_mask = enabled);
}

fn depth_range(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let near = a.get(0).number_value(s).unwrap_or(0.0).clamp(0.0, 1.0);
    let far = a.get(1).number_value(s).unwrap_or(1.0).clamp(0.0, 1.0);
    update(s, a.this(), |context| context.depth_range = [near, far]);
}

fn line_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = a.get(0).number_value(s).unwrap_or(1.0);
    update(s, a.this(), |context| {
        if width <= 0.0 || !width.is_finite() {
            set_error(context, INVALID_VALUE);
        } else {
            context.line_width = width;
        }
    });
}

fn pixel_store_i(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let parameter = a.get(0).uint32_value(s).unwrap_or(0);
    let value = a.get(1).int32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.pixel_store.insert(parameter, value);
    });
}

fn polygon_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let factor = a.get(0).number_value(s).unwrap_or(0.0);
    let units = a.get(1).number_value(s).unwrap_or(0.0);
    update(s, a.this(), |context| {
        context.polygon_offset = [factor, units]
    });
}

fn sample_coverage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(s).unwrap_or(1.0).clamp(0.0, 1.0);
    let invert = if a.get(1).boolean_value(s) { 1.0 } else { 0.0 };
    update(s, a.this(), |context| {
        context.sample_coverage = [value, invert]
    });
}

fn color_mask(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mask = [
        a.get(0).boolean_value(s),
        a.get(1).boolean_value(s),
        a.get(2).boolean_value(s),
        a.get(3).boolean_value(s),
    ];
    update(s, a.this(), |context| context.color_mask = mask);
}

fn scissor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = [
        a.get(0).int32_value(s).unwrap_or(0),
        a.get(1).int32_value(s).unwrap_or(0),
        a.get(2).int32_value(s).unwrap_or(0),
        a.get(3).int32_value(s).unwrap_or(0),
    ];
    update(s, a.this(), |context| {
        if value[2] < 0 || value[3] < 0 {
            set_error(context, INVALID_VALUE);
        } else {
            context.scissor = value;
        }
    });
}

fn hint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let mode = a.get(1).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.hints.insert(target, mode);
    });
}

fn enable_vertex_attrib_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.vertex_attrib_enabled.insert(index);
    });
}

fn disable_vertex_attrib_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.vertex_attrib_enabled.remove(&index);
    });
}

fn vertex_attrib_pointer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = a.get(0).uint32_value(s).unwrap_or(0);
    let size = a.get(1).int32_value(s).unwrap_or(0);
    let kind = a.get(2).uint32_value(s).unwrap_or(0);
    let normalized = a.get(3).boolean_value(s);
    let stride = a.get(4).int32_value(s).unwrap_or(0);
    let offset = a.get(5).int32_value(s).unwrap_or(0);
    let buffer = record(s, a.this()).and_then(|context| context.bound_array_buffer);
    update(s, a.this(), |context| {
        if !(1..=4).contains(&size) || stride < 0 || offset < 0 || buffer.is_none() {
            set_error(context, INVALID_VALUE);
        } else {
            context.vertex_attrib_pointers.insert(
                index,
                VertexAttribRecord {
                    size,
                    kind,
                    normalized,
                    stride,
                    offset,
                    buffer,
                },
            );
        }
    });
}

fn set_vertex_attrib(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    component_count: usize,
    vector_argument: bool,
) {
    let index = a.get(0).uint32_value(s).unwrap_or(0);
    let mut value = [0.0, 0.0, 0.0, 1.0];
    if vector_argument {
        let values = numeric_values(s, a.get(1), component_count);
        for component in 0..component_count {
            value[component] = values.get(component).copied().unwrap_or(0.0);
        }
    } else {
        for component in 0..component_count {
            value[component] = a.get((component + 1) as i32).number_value(s).unwrap_or(0.0);
        }
    }
    update(s, a.this(), |context| {
        context.vertex_attrib_values.insert(index, value);
    });
}

fn vertex_attrib_1f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 1, false);
}
fn vertex_attrib_1fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 1, true);
}
fn vertex_attrib_2f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 2, false);
}
fn vertex_attrib_2fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 2, true);
}
fn vertex_attrib_3f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 3, false);
}
fn vertex_attrib_3fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 3, true);
}
fn vertex_attrib_4f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 4, false);
}
fn vertex_attrib_4fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_vertex_attrib(s, a, 4, true);
}

fn get_vertex_attrib(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let index = a.get(0).uint32_value(s).unwrap_or(0);
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    let Some(context) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let pointer = context.vertex_attrib_pointers.get(&index);
    match parameter {
        0x8622 => r.set(v8::Boolean::new(s, context.vertex_attrib_enabled.contains(&index)).into()),
        0x8623 => r.set(v8::Integer::new(s, pointer.map_or(4, |pointer| pointer.size)).into()),
        0x8624 => r.set(v8::Integer::new(s, pointer.map_or(0, |pointer| pointer.stride)).into()),
        0x8625 => r.set(
            v8::Integer::new_from_unsigned(s, pointer.map_or(0x1406, |pointer| pointer.kind))
                .into(),
        ),
        0x886A => {
            r.set(v8::Boolean::new(s, pointer.is_some_and(|pointer| pointer.normalized)).into())
        }
        0x889F => {
            if let Some(buffer) = pointer.and_then(|pointer| pointer.buffer.as_ref()) {
                r.set(v8::Local::new(s, buffer).into());
            } else {
                r.set(v8::null(s).into());
            }
        }
        0x8626 => {
            let values = context
                .vertex_attrib_values
                .get(&index)
                .copied()
                .unwrap_or([0.0, 0.0, 0.0, 1.0]);
            let array = v8::Array::new(s, 4);
            for (component, value) in values.iter().enumerate() {
                let number = v8::Number::new(s, *value);
                let _ = array.set_index(s, component as u32, number.into());
            }
            r.set(array.into());
        }
        _ => r.set(v8::null(s).into()),
    }
}

fn get_vertex_attrib_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let index = a.get(0).uint32_value(s).unwrap_or(0);
    let offset = record(s, a.this())
        .and_then(|context| {
            context
                .vertex_attrib_pointers
                .get(&index)
                .map(|pointer| pointer.offset)
        })
        .unwrap_or(0);
    r.set(v8::Integer::new(s, offset).into());
}

fn append_declarations(source: &str, qualifier: &str, output: &mut Vec<String>) {
    for statement in source.split(';') {
        let mut words = statement.split_whitespace();
        let Some(first) = words.next() else {
            continue;
        };
        if first != qualifier {
            continue;
        }
        let _kind = words.next();
        let Some(raw_name) = words.next() else {
            continue;
        };
        let name = raw_name
            .split('[')
            .next()
            .unwrap_or(raw_name)
            .trim()
            .to_owned();
        if !name.is_empty() && !output.iter().any(|existing| existing == &name) {
            output.push(name);
        }
    }
}

fn numeric_values(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    maximum: usize,
) -> Vec<f64> {
    if value.is_number() {
        return vec![value.number_value(scope).unwrap_or(0.0)];
    }
    let Some(object) = v8::Local::<v8::Object>::try_from(value).ok() else {
        return Vec::new();
    };
    let length = v8::String::new(scope, "length")
        .and_then(|key| object.get(scope, key.into()))
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0)
        .min(maximum as u32);
    let mut values = Vec::with_capacity(length as usize);
    for index in 0..length {
        let value = object
            .get_index(scope, index)
            .and_then(|value| value.number_value(scope))
            .unwrap_or(0.0);
        values.push(value);
    }
    values
}

fn set_uniform_values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    values: Vec<f64>,
) {
    if a.get(0).is_null() {
        return;
    }
    let location = object_id(a.get(0));
    update(s, a.this(), |context| {
        let Some(location) = location.and_then(|id| context.uniform_locations.get_mut(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        location.values = values;
    });
}

fn scalar_uniform_values(
    s: &mut v8::PinScope<'_, '_>,
    a: &v8::FunctionCallbackArguments<'_>,
    component_count: usize,
) -> Vec<f64> {
    let mut values = Vec::with_capacity(component_count);
    for component in 0..component_count {
        values.push(a.get((component + 1) as i32).number_value(s).unwrap_or(0.0));
    }
    values
}

fn uniform_1f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 1);
    set_uniform_values(s, a, values);
}
fn uniform_1i(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 1);
    set_uniform_values(s, a, values);
}
fn uniform_2f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 2);
    set_uniform_values(s, a, values);
}
fn uniform_2i(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 2);
    set_uniform_values(s, a, values);
}
fn uniform_3f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 3);
    set_uniform_values(s, a, values);
}
fn uniform_3i(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 3);
    set_uniform_values(s, a, values);
}
fn uniform_4f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 4);
    set_uniform_values(s, a, values);
}
fn uniform_4i(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = scalar_uniform_values(s, &a, 4);
    set_uniform_values(s, a, values);
}

fn vector_uniform(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>) {
    let values = numeric_values(s, a.get(1), 16_384);
    set_uniform_values(s, a, values);
}

fn uniform_1fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_1iv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_2fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_2iv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_3fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_3iv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_4fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}
fn uniform_4iv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    vector_uniform(s, a)
}

fn matrix_uniform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    matrix_size: usize,
) {
    if a.get(1).boolean_value(s) {
        update(s, a.this(), |context| set_error(context, INVALID_VALUE));
        return;
    }
    let values = numeric_values(s, a.get(2), matrix_size.saturating_mul(4_096));
    if values.len() % matrix_size != 0 {
        update(s, a.this(), |context| set_error(context, INVALID_VALUE));
        return;
    }
    set_uniform_values(s, a, values);
}

fn uniform_matrix_2fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    matrix_uniform(s, a, 4)
}
fn uniform_matrix_3fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    matrix_uniform(s, a, 9)
}
fn uniform_matrix_4fv(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    matrix_uniform(s, a, 16)
}

fn get_uniform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let program = object_id(a.get(0));
    let location = object_id(a.get(1));
    let uniform = location.and_then(|id| record(s, a.this())?.uniform_locations.get(&id).cloned());
    let Some(uniform) = uniform.filter(|uniform| Some(uniform.program) == program) else {
        r.set(v8::null(s).into());
        return;
    };
    if uniform.values.len() == 1 {
        r.set(v8::Number::new(s, uniform.values[0]).into());
        return;
    }
    let array = v8::Array::new(s, uniform.values.len() as i32);
    for (index, value) in uniform.values.iter().enumerate() {
        let value = v8::Number::new(s, *value);
        let _ = array.set_index(s, index as u32, value.into());
    }
    r.set(array.into());
}

fn compressed_tex_image_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let format = a.get(2).uint32_value(s).unwrap_or(0);
    let width = a.get(3).uint32_value(s).unwrap_or(0);
    let height = a.get(4).uint32_value(s).unwrap_or(0);
    update_texture_image(s, a.this(), target, format, width, height);
}

fn compressed_tex_sub_image_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let width = a.get(4).uint32_value(s).unwrap_or(0);
    let height = a.get(5).uint32_value(s).unwrap_or(0);
    validate_texture_sub_image(s, a.this(), target, width, height);
}

fn copy_tex_image_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let format = a.get(2).uint32_value(s).unwrap_or(0);
    let width = a.get(5).uint32_value(s).unwrap_or(0);
    let height = a.get(6).uint32_value(s).unwrap_or(0);
    update_texture_image(s, a.this(), target, format, width, height);
}

fn copy_tex_sub_image_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let width = a.get(6).uint32_value(s).unwrap_or(0);
    let height = a.get(7).uint32_value(s).unwrap_or(0);
    validate_texture_sub_image(s, a.this(), target, width, height);
}

fn tex_image_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let format = a.get(2).uint32_value(s).unwrap_or(0);
    let width = a.get(3).uint32_value(s).unwrap_or(0);
    let height = a.get(4).uint32_value(s).unwrap_or(0);
    update_texture_image(s, a.this(), target, format, width, height);
}

fn tex_sub_image_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let width = a.get(4).uint32_value(s).unwrap_or(0);
    let height = a.get(5).uint32_value(s).unwrap_or(0);
    validate_texture_sub_image(s, a.this(), target, width, height);
}

fn update_texture_image(
    s: &mut v8::PinScope<'_, '_>,
    context_object: v8::Local<'_, v8::Object>,
    target: u32,
    format: u32,
    width: u32,
    height: u32,
) {
    let texture = bound_texture_id(s, context_object, target);
    update(s, context_object, |context| {
        let Some(texture) = texture.and_then(|id| context.textures.get_mut(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        texture.width = width;
        texture.height = height;
        texture.format = format;
        texture.mipmaps_generated = false;
    });
}

fn validate_texture_sub_image(
    s: &mut v8::PinScope<'_, '_>,
    context_object: v8::Local<'_, v8::Object>,
    target: u32,
    width: u32,
    height: u32,
) {
    let texture = bound_texture_id(s, context_object, target);
    update(s, context_object, |context| {
        let Some(texture) = texture.and_then(|id| context.textures.get(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        if width > texture.width || height > texture.height {
            set_error(context, INVALID_VALUE);
        }
    });
}

fn set_texture_parameter(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    let value = a.get(2).number_value(s).unwrap_or(0.0);
    let texture = bound_texture_id(s, a.this(), target);
    update(s, a.this(), |context| {
        let Some(texture) = texture.and_then(|id| context.textures.get_mut(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        texture.parameters.insert(parameter, value);
    });
}

fn tex_parameter_f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_texture_parameter(s, a);
}

fn tex_parameter_i(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_texture_parameter(s, a);
}

fn get_tex_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    let value = bound_texture_id(s, a.this(), target).and_then(|texture| {
        record(s, a.this())?
            .textures
            .get(&texture)?
            .parameters
            .get(&parameter)
            .copied()
    });
    match value {
        Some(value) => r.set(v8::Number::new(s, value).into()),
        None => r.set(v8::null(s).into()),
    }
}

fn generate_mipmap(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let texture = bound_texture_id(s, a.this(), target);
    update(s, a.this(), |context| {
        let Some(texture) = texture.and_then(|id| context.textures.get_mut(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        if texture.width == 0 || texture.height == 0 {
            set_error(context, INVALID_OPERATION);
        } else {
            texture.mipmaps_generated = true;
        }
    });
}

fn renderbuffer_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let internal_format = a.get(1).uint32_value(s).unwrap_or(0);
    let width = a.get(2).uint32_value(s).unwrap_or(0);
    let height = a.get(3).uint32_value(s).unwrap_or(0);
    let renderbuffer = record(s, a.this())
        .and_then(|context| global_object_id(s, context.bound_renderbuffer.as_ref()));
    update(s, a.this(), |context| {
        if target != 0x8D41 {
            set_error(context, INVALID_ENUM);
            return;
        }
        let Some(renderbuffer) = renderbuffer.and_then(|id| context.renderbuffers.get_mut(&id))
        else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        renderbuffer.internal_format = internal_format;
        renderbuffer.width = width;
        renderbuffer.height = height;
    });
}

fn get_renderbuffer_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let parameter = a.get(1).uint32_value(s).unwrap_or(0);
    let value = record(s, a.this()).and_then(|context| {
        let id = global_object_id(s, context.bound_renderbuffer.as_ref())?;
        let renderbuffer = context.renderbuffers.get(&id)?;
        match parameter {
            0x8D42 => Some(renderbuffer.width),
            0x8D43 => Some(renderbuffer.height),
            0x8D44 => Some(renderbuffer.internal_format),
            _ => Some(0),
        }
    });
    if let Some(value) = value {
        r.set(v8::Integer::new_from_unsigned(s, value).into());
    } else {
        r.set(v8::null(s).into());
    }
}

fn framebuffer_renderbuffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let attachment = a.get(1).uint32_value(s).unwrap_or(0);
    let renderbuffer = object_id(a.get(3));
    update_framebuffer_attachment(s, a.this(), target, attachment, renderbuffer);
}

fn framebuffer_texture_2d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = a.get(0).uint32_value(s).unwrap_or(0);
    let attachment = a.get(1).uint32_value(s).unwrap_or(0);
    let texture = object_id(a.get(3));
    update_framebuffer_attachment(s, a.this(), target, attachment, texture);
}

fn update_framebuffer_attachment(
    s: &mut v8::PinScope<'_, '_>,
    context_object: v8::Local<'_, v8::Object>,
    target: u32,
    attachment: u32,
    resource: Option<i32>,
) {
    let framebuffer = record(s, context_object)
        .and_then(|context| global_object_id(s, context.bound_framebuffer.as_ref()));
    update(s, context_object, |context| {
        if target != 0x8D40 {
            set_error(context, INVALID_ENUM);
            return;
        }
        let Some(framebuffer) = framebuffer.and_then(|id| context.framebuffers.get_mut(&id)) else {
            set_error(context, INVALID_OPERATION);
            return;
        };
        if let Some(resource) = resource {
            framebuffer.attachments.insert(attachment, resource);
        } else {
            framebuffer.attachments.remove(&attachment);
        }
    });
}

fn get_framebuffer_attachment_parameter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let attachment = a.get(1).uint32_value(s).unwrap_or(0);
    let parameter = a.get(2).uint32_value(s).unwrap_or(0);
    let resource = record(s, a.this()).and_then(|context| {
        let framebuffer = global_object_id(s, context.bound_framebuffer.as_ref())?;
        context
            .framebuffers
            .get(&framebuffer)?
            .attachments
            .get(&attachment)
            .copied()
    });
    match (parameter, resource) {
        (0x8CD0, Some(resource)) => {
            let kind = record(s, a.this()).map_or(0, |context| {
                if context.textures.contains_key(&resource) {
                    0x1702
                } else if context.renderbuffers.contains_key(&resource) {
                    0x8D41
                } else {
                    0
                }
            });
            r.set(v8::Integer::new_from_unsigned(s, kind).into());
        }
        (_, Some(resource)) => {
            let object = record(s, a.this()).and_then(|context| {
                if let Some(texture) = context.bound_texture_2d.as_ref() {
                    let local = v8::Local::new(s, texture);
                    (local.get_identity_hash().get() == resource).then_some(local)
                } else {
                    None
                }
            });
            if let Some(object) = object {
                r.set(object.into());
            } else {
                r.set(v8::null(s).into());
            }
        }
        _ => r.set(v8::null(s).into()),
    }
}

fn read_pixels(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = a.get(2).int32_value(s).unwrap_or(0);
    let height = a.get(3).int32_value(s).unwrap_or(0);
    let destination = v8::Local::<v8::Object>::try_from(a.get(6)).ok();
    let clear = record(s, a.this()).map(|context| context.clear_color);
    let Some(destination) = destination else {
        update(s, a.this(), |context| set_error(context, INVALID_VALUE));
        return;
    };
    if width < 0 || height < 0 {
        update(s, a.this(), |context| set_error(context, INVALID_VALUE));
        return;
    }
    let clear = clear.unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let pixel_count = (width as u32).saturating_mul(height as u32);
    for pixel in 0..pixel_count {
        for channel in 0..4_u32 {
            let byte = (clear[channel as usize].clamp(0.0, 1.0) * 255.0).round() as u32;
            let value = v8::Integer::new_from_unsigned(s, byte);
            let _ = destination.set_index(s, pixel.saturating_mul(4) + channel, value.into());
        }
    }
}

fn stencil_func(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let function = a.get(0).uint32_value(s).unwrap_or(0);
    let reference = a.get(1).uint32_value(s).unwrap_or(0);
    let mask = a.get(2).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.stencil_front[0] = function;
        context.stencil_front[1] = reference;
        context.stencil_front[2] = mask;
        context.stencil_back[0] = function;
        context.stencil_back[1] = reference;
        context.stencil_back[2] = mask;
    });
}

fn stencil_func_separate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let face = a.get(0).uint32_value(s).unwrap_or(0);
    let values = [
        a.get(1).uint32_value(s).unwrap_or(0),
        a.get(2).uint32_value(s).unwrap_or(0),
        a.get(3).uint32_value(s).unwrap_or(0),
    ];
    update(s, a.this(), |context| {
        if face == 0x0404 || face == 0x0408 {
            context.stencil_front[0..3].copy_from_slice(&values);
        }
        if face == 0x0405 || face == 0x0408 {
            context.stencil_back[0..3].copy_from_slice(&values);
        }
    });
}

fn stencil_mask(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mask = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        context.stencil_front[3] = mask;
        context.stencil_back[3] = mask;
    });
}

fn stencil_mask_separate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let face = a.get(0).uint32_value(s).unwrap_or(0);
    let mask = a.get(1).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |context| {
        if face == 0x0404 || face == 0x0408 {
            context.stencil_front[3] = mask;
        }
        if face == 0x0405 || face == 0x0408 {
            context.stencil_back[3] = mask;
        }
    });
}

fn stencil_op(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let values = [
        a.get(0).uint32_value(s).unwrap_or(0),
        a.get(1).uint32_value(s).unwrap_or(0),
        a.get(2).uint32_value(s).unwrap_or(0),
    ];
    update(s, a.this(), |context| {
        context.stencil_front[4..7].copy_from_slice(&values);
        context.stencil_back[4..7].copy_from_slice(&values);
    });
}

fn stencil_op_separate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let face = a.get(0).uint32_value(s).unwrap_or(0);
    let values = [
        a.get(1).uint32_value(s).unwrap_or(0),
        a.get(2).uint32_value(s).unwrap_or(0),
        a.get(3).uint32_value(s).unwrap_or(0),
    ];
    update(s, a.this(), |context| {
        if face == 0x0404 || face == 0x0408 {
            context.stencil_front[4..7].copy_from_slice(&values);
        }
        if face == 0x0405 || face == 0x0408 {
            context.stencil_back[4..7].copy_from_slice(&values);
        }
    });
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

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebGlRenderingContextStore>() {
        store.constructors.remove(&realm_id);
    }
}
