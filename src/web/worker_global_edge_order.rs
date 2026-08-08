pub(crate) fn install_dedicated(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    super::worker_global_scope::mirror_global_to_target(scope, target, "console")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebSocketStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebSocketError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "RestrictionTarget")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "RTCTransformEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "RTCRtpScriptTransformer")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "RTCDataChannel")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "QuotaExceededError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PushSubscriptionOptions")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PushSubscription")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PushManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PeriodicSyncManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Origin")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Notification")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CropTarget")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "BackgroundFetchRegistration",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "BackgroundFetchRecord")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "BackgroundFetchManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "XMLHttpRequestUpload")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "XMLHttpRequestEventTarget",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "XMLHttpRequest")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "WritableStreamDefaultWriter",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "WritableStreamDefaultController",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WritableStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WorkerNavigator")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WorkerLocation")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WorkerGlobalScope")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Worker")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebSocket")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLVertexArrayObject")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLUniformLocation")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLTransformFeedback")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLTexture")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLSync")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "WebGLShaderPrecisionFormat",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLShader")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLSampler")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLRenderingContext")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLRenderbuffer")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLQuery")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLProgram")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLObject")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLFramebuffer")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLContextEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLBuffer")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGLActiveInfo")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebGL2RenderingContext")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "VideoFrame")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "VideoColorSpace")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "UserActivation")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "URLSearchParams")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "URLPattern")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "URL")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TrustedTypePolicyFactory")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TrustedTypePolicy")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TrustedScriptURL")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TrustedScript")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TrustedHTML")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "TransformStreamDefaultController",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TransformStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TextMetrics")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TextEncoderStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TextEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TextDecoderStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TextDecoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TaskSignal")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TaskPriorityChangeEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "TaskController")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "SyncManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Subscriber")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "SourceBufferList")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "SourceBuffer")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "SecurityPolicyViolationEvent",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Scheduler")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Response")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Request")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ReportingObserver")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ReportBody")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ReadableStreamDefaultReader",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ReadableStreamDefaultController",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ReadableStreamBYOBRequest",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ReadableStreamBYOBReader")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ReadableStream")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ReadableByteStreamController",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "RTCEncodedVideoFrame")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "RTCEncodedAudioFrame")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PromiseRejectionEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ProgressEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Permissions")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PermissionStatus")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PerformanceServerTiming")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "PerformanceResourceTiming",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "PerformanceObserverEntryList",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PerformanceObserver")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PerformanceMeasure")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PerformanceMark")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PerformanceEntry")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Performance")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Path2D")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "OffscreenCanvasRenderingContext2D",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "OffscreenCanvas")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Observable")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "NetworkInformation")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "NavigatorUAData")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "MessagePort")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "MessageEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "MessageChannel")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "MediaSourceHandle")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "MediaSource")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "MediaCapabilities")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ImageData")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ImageBitmapRenderingContext",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ImageBitmap")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBVersionChangeEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBTransaction")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBRequest")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBRecord")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBOpenDBRequest")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBObjectStore")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBKeyRange")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBIndex")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBFactory")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBDatabase")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBCursorWithValue")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IDBCursor")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Headers")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FormData")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FontFace")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FileReaderSync")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FileReader")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FileList")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "File")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "EventTarget")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "EventSource")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Event")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ErrorEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "EncodedVideoChunk")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "EncodedAudioChunk")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "DedicatedWorkerGlobalScope",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DecompressionStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMStringList")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMRectReadOnly")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMRect")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMQuad")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMPointReadOnly")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMPoint")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMMatrixReadOnly")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMMatrix")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DOMException")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CustomEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Crypto")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CountQueuingStrategy")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CompressionStream")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CloseEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CanvasPattern")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CanvasGradient")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CSSSkewY")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CSSSkewX")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ByteLengthQueuingStrategy",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "BroadcastChannel")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Blob")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "AudioData")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "AbortSignal")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "AbortController")?;
    super::worker_global_scope::install_dedicated_global_members(scope, target)?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Temporal")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "SuppressedError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "DisposableStack")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "AsyncDisposableStack")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Float16Array")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebAssembly")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "AudioDecoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "AudioEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Cache")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CacheStorage")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CreateMonitor")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "CryptoKey")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "FileSystemSyncAccessHandle",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPU")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUAdapter")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUAdapterInfo")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUBindGroup")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUBindGroupLayout")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUBuffer")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUBufferUsage")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUCanvasContext")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUColorWrite")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUCommandBuffer")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUCommandEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUCompilationInfo")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUCompilationMessage")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUComputePassEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUComputePipeline")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUDevice")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUDeviceLostInfo")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUExternalTexture")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUInternalError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUMapMode")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUOutOfMemoryError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUPipelineError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUPipelineLayout")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUQuerySet")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUQueue")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPURenderBundle")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPURenderBundleEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPURenderPassEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPURenderPipeline")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUSampler")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUShaderModule")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUShaderStage")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUSupportedFeatures")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUSupportedLimits")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUTexture")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUTextureUsage")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUTextureView")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUUncapturedErrorEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "GPUValidationError")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "IdleDetector")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ImageDecoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ImageTrack")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "ImageTrackList")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "NavigationPreloadManager")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "ServiceWorkerRegistration",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "StorageManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "SubtleCrypto")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "VideoDecoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "VideoEncoder")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WGSLLanguageFeatures")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebTransport")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "WebTransportBidirectionalStream",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "WebTransportDatagramDuplexStream",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "WebTransportError")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "FileSystemDirectoryHandle",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FileSystemFileHandle")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FileSystemHandle")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "FileSystemWritableFileStream",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "FileSystemObserver")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "HID")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "HIDConnectionEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "HIDDevice")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "HIDInputReportEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Lock")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "LockManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PressureObserver")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "PressureRecord")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "Serial")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "SerialPort")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "StorageBucket")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "StorageBucketManager")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USB")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBAlternateInterface")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBConfiguration")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBConnectionEvent")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBDevice")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBEndpoint")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBInTransferResult")?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBInterface")?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "USBIsochronousInTransferPacket",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "USBIsochronousInTransferResult",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "USBIsochronousOutTransferPacket",
    )?;
    super::worker_global_scope::mirror_global_to_target(
        scope,
        target,
        "USBIsochronousOutTransferResult",
    )?;
    super::worker_global_scope::mirror_global_to_target(scope, target, "USBOutTransferResult")
}
