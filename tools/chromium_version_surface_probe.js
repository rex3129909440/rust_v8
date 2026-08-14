(async function collectChromiumVersionSurface() {
  "use strict";

  function keyName(key) {
    return typeof key === "symbol" ? "@@" + String(key.description || "") : key;
  }

  function functionShape(value) {
    if (typeof value !== "function") return null;
    var source = "";
    try {
      source = Function.prototype.toString.call(value);
    } catch (_) {}
    return {
      name: value.name,
      length: value.length,
      native: source.indexOf("[native code]") !== -1,
    };
  }

  function descriptorShape(object, key) {
    var descriptor;
    try {
      descriptor = Object.getOwnPropertyDescriptor(object, key);
    } catch (error) {
      return { error: String(error && error.name || error) };
    }
    if (!descriptor) return null;
    var result = {
      configurable: descriptor.configurable,
      enumerable: descriptor.enumerable,
    };
    if (Object.prototype.hasOwnProperty.call(descriptor, "value")) {
      result.kind = "data";
      result.writable = descriptor.writable;
      result.valueType = typeof descriptor.value;
      result.function = functionShape(descriptor.value);
    } else {
      result.kind = "accessor";
      result.get = functionShape(descriptor.get);
      result.set = functionShape(descriptor.set);
    }
    return result;
  }

  function ownSurface(object) {
    var names = Object.getOwnPropertyNames(object);
    var keys = Reflect.ownKeys(object);
    var descriptors = {};
    for (var index = 0; index < keys.length; index += 1) {
      var key = keys[index];
      descriptors[keyName(key)] = descriptorShape(object, key);
    }
    return {
      names: names,
      keys: keys.map(keyName),
      descriptors: descriptors,
    };
  }

  function constructorPrototypeSurfaces(globalObject) {
    var result = {};
    var names = Object.getOwnPropertyNames(globalObject);
    for (var index = 0; index < names.length; index += 1) {
      var name = names[index];
      var descriptor = Object.getOwnPropertyDescriptor(globalObject, name);
      if (!descriptor || !("value" in descriptor) || typeof descriptor.value !== "function") {
        continue;
      }
      var prototype;
      try {
        prototype = descriptor.value.prototype;
      } catch (_) {
        continue;
      }
      if (!prototype || (typeof prototype !== "object" && typeof prototype !== "function")) {
        continue;
      }
      result[name] = ownSurface(prototype);
    }
    return result;
  }

  function constructorStaticSurfaces(globalObject) {
    var result = {};
    var names = Object.getOwnPropertyNames(globalObject);
    for (var index = 0; index < names.length; index += 1) {
      var name = names[index];
      var descriptor = Object.getOwnPropertyDescriptor(globalObject, name);
      if (descriptor && "value" in descriptor && typeof descriptor.value === "function") {
        result[name] = ownSurface(descriptor.value);
      }
    }
    return result;
  }

  function globalObjectSurfaces(globalObject) {
    var result = {};
    var names = Object.getOwnPropertyNames(globalObject);
    for (var index = 0; index < names.length; index += 1) {
      var name = names[index];
      var descriptor = Object.getOwnPropertyDescriptor(globalObject, name);
      if (!descriptor || !("value" in descriptor)) continue;
      var value = descriptor.value;
      if (!value || typeof value !== "object" || value === globalObject) continue;
      var ownNames = Object.getOwnPropertyNames(value);
      if (ownNames.length) result[name] = ownSurface(value);
    }
    return result;
  }

  function resultOf(callback) {
    try {
      var value = callback();
      return { ok: true, value: value };
    } catch (error) {
      return {
        ok: false,
        error: String(error && error.name || error),
        message: String(error && error.message || ""),
      };
    }
  }

  async function promiseResult(callback) {
    try {
      return { ok: true, value: await callback() };
    } catch (error) {
      return {
        ok: false,
        error: String(error && error.name || error),
        message: String(error && error.message || ""),
      };
    }
  }

  function shallowValues(object, names) {
    var result = {};
    for (var index = 0; index < names.length; index += 1) {
      var name = names[index];
      result[name] = resultOf(function () { return object[name]; });
    }
    return result;
  }

  function entryShape(entry) {
    var json = resultOf(function () { return entry.toJSON(); });
    var chain = [], current = entry;
    while (current !== null && chain.length < 8) {
      chain.push(ownSurface(current));
      current = Object.getPrototypeOf(current);
    }
    return {
      constructor: resultOf(function () { return entry.constructor.name; }),
      entryType: resultOf(function () { return entry.entryType; }),
      own: ownSurface(entry),
      chain: chain,
      jsonKeys: json.ok && json.value && typeof json.value === "object"
        ? Object.keys(json.value) : [],
      json: json,
    };
  }

  async function collectRuntimeEvidence() {
    var audio = document.createElement("audio");
    var video = document.createElement("video");
    var contentTypes = [
      'audio/mpeg',
      'audio/mp4; codecs="mp4a.40.2"',
      'audio/ogg; codecs="vorbis"',
      'audio/webm; codecs="opus"',
      'video/mp4; codecs="avc1.42E01E"',
      'video/mp4; codecs="hvc1.1.6.L93.B0"',
      'video/webm; codecs="vp8"',
      'video/webm; codecs="vp09.00.10.08"',
      'video/webm; codecs="av01.0.04M.08"',
    ];
    var media = { canPlayType: {}, mediaSource: {}, mediaRecorder: {} };
    for (var index = 0; index < contentTypes.length; index += 1) {
      var contentType = contentTypes[index];
      media.canPlayType[contentType] = {
        audio: resultOf(function (type) {
          return function () { return audio.canPlayType(type); };
        }(contentType)),
        video: resultOf(function (type) {
          return function () { return video.canPlayType(type); };
        }(contentType)),
      };
      media.mediaSource[contentType] = resultOf(function (type) {
        return function () {
          return typeof MediaSource === "function" && MediaSource.isTypeSupported(type);
        };
      }(contentType));
      media.mediaRecorder[contentType] = resultOf(function (type) {
        return function () {
          return typeof MediaRecorder === "function" && MediaRecorder.isTypeSupported(type);
        };
      }(contentType));
    }

    media.supportedConstraints = resultOf(function () {
      return navigator.mediaDevices.getSupportedConstraints();
    });
    media.enumerateDevices = await promiseResult(async function () {
      var devices = await navigator.mediaDevices.enumerateDevices();
      return devices.map(function (device) {
        return {
          kind: device.kind,
          hasDeviceId: Boolean(device.deviceId),
          hasGroupId: Boolean(device.groupId),
          hasLabel: Boolean(device.label),
          own: ownSurface(device),
          prototype: ownSurface(Object.getPrototypeOf(device)),
        };
      });
    });

    media.decodingInfo = {};
    if (navigator.mediaCapabilities) {
      var decodingConfigurations = {
        aac: { type: "file", audio: { contentType: 'audio/mp4; codecs="mp4a.40.2"', channels: "2", bitrate: 128000, samplerate: 48000 } },
        opus: { type: "file", audio: { contentType: 'audio/webm; codecs="opus"', channels: "2", bitrate: 128000, samplerate: 48000 } },
        h264: { type: "file", video: { contentType: 'video/mp4; codecs="avc1.42E01E"', width: 1920, height: 1080, bitrate: 5000000, framerate: 30 } },
        vp9: { type: "file", video: { contentType: 'video/webm; codecs="vp09.00.10.08"', width: 1920, height: 1080, bitrate: 5000000, framerate: 30 } },
        av1: { type: "file", video: { contentType: 'video/webm; codecs="av01.0.04M.08"', width: 1920, height: 1080, bitrate: 5000000, framerate: 30 } },
      };
      var decodingNames = Object.keys(decodingConfigurations);
      for (var decodingIndex = 0; decodingIndex < decodingNames.length; decodingIndex += 1) {
        var decodingName = decodingNames[decodingIndex];
        media.decodingInfo[decodingName] = await promiseResult(function (configuration) {
          return function () { return navigator.mediaCapabilities.decodingInfo(configuration); };
        }(decodingConfigurations[decodingName]));
      }
    }

    media.audioContext = resultOf(function () {
      var Constructor = window.AudioContext || window.webkitAudioContext;
      if (!Constructor) return null;
      var context = new Constructor();
      var result = {
        sampleRate: context.sampleRate,
        baseLatency: context.baseLatency,
        outputLatency: context.outputLatency,
        destination: shallowValues(context.destination, [
          "channelCount", "channelCountMode", "channelInterpretation", "maxChannelCount"
        ]),
      };
      context.close();
      return result;
    });

    var queries = [
      "(any-hover: hover)", "(any-pointer: coarse)", "(hover: hover)",
      "(pointer: coarse)", "(display-mode: browser)",
      "(orientation: portrait)", "(prefers-reduced-motion: reduce)",
      "(prefers-color-scheme: dark)", "(dynamic-range: high)",
      "(video-dynamic-range: high)", "(device-width: 393px)",
    ];
    var mediaQueries = {};
    for (var queryIndex = 0; queryIndex < queries.length; queryIndex += 1) {
      var query = queries[queryIndex];
      mediaQueries[query] = resultOf(function (value) {
        return function () {
          var list = matchMedia(value);
          return { matches: list.matches, media: list.media };
        };
      }(query));
    }

    var webgl = resultOf(function () {
      var canvas = document.createElement("canvas");
      var gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
      if (!gl) return null;
      var debug = gl.getExtension("WEBGL_debug_renderer_info");
      return {
        constructor: gl.constructor.name,
        version: gl.getParameter(gl.VERSION),
        shadingLanguageVersion: gl.getParameter(gl.SHADING_LANGUAGE_VERSION),
        vendor: gl.getParameter(gl.VENDOR),
        renderer: gl.getParameter(gl.RENDERER),
        unmaskedVendor: debug ? gl.getParameter(debug.UNMASKED_VENDOR_WEBGL) : null,
        unmaskedRenderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null,
        maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE),
        maxRenderbufferSize: gl.getParameter(gl.MAX_RENDERBUFFER_SIZE),
        maxViewportDims: Array.from(gl.getParameter(gl.MAX_VIEWPORT_DIMS)),
        extensions: gl.getSupportedExtensions(),
      };
    });

    var entries = performance.getEntries();
    var observerTypes = resultOf(function () {
      return typeof PerformanceObserver === "function"
        ? PerformanceObserver.supportedEntryTypes.slice() : [];
    });
    var connection = navigator.connection || navigator.mozConnection || navigator.webkitConnection;
    var orientation = screen.orientation;
    return {
      secureContext: isSecureContext,
      crossOriginIsolated: crossOriginIsolated,
      document: shallowValues(document, ["visibilityState", "hidden", "hasFocus"]),
      navigator: shallowValues(navigator, [
        "platform", "product", "vendor", "hardwareConcurrency", "deviceMemory",
        "maxTouchPoints", "webdriver", "pdfViewerEnabled", "cookieEnabled",
      ]),
      connection: connection ? shallowValues(connection, [
        "downlink", "effectiveType", "rtt", "saveData", "type"
      ]) : null,
      screen: shallowValues(screen, [
        "width", "height", "availWidth", "availHeight", "colorDepth", "pixelDepth",
        "availLeft", "availTop", "isExtended"
      ]),
      orientation: orientation ? shallowValues(orientation, ["type", "angle"]) : null,
      window: shallowValues(window, [
        "innerWidth", "innerHeight", "outerWidth", "outerHeight", "devicePixelRatio",
        "screenX", "screenY", "scrollX", "scrollY"
      ]),
      visualViewport: visualViewport ? shallowValues(visualViewport, [
        "width", "height", "offsetLeft", "offsetTop", "pageLeft", "pageTop", "scale"
      ]) : null,
      mediaQueries: mediaQueries,
      media: media,
      webgl: webgl,
      performance: {
        instance: ownSurface(performance),
        prototype: ownSurface(Object.getPrototypeOf(performance)),
        timeOrigin: performance.timeOrigin,
        nowSamples: [performance.now(), performance.now(), performance.now()],
        observerSupportedEntryTypes: observerTypes,
        entries: entries.map(entryShape),
        timing: resultOf(function () { return performance.timing.toJSON(); }),
        navigation: resultOf(function () { return performance.navigation.toJSON(); }),
        memory: resultOf(function () { return Object.assign({}, performance.memory); }),
        eventCounts: resultOf(function () {
          return performance.eventCounts
            ? Array.from(performance.eventCounts.entries()) : null;
        }),
      },
    };
  }

  function workerSource() {
    return `
      (function () {
        function keyName(key) {
          return typeof key === "symbol" ? "@@" + String(key.description || "") : key;
        }
        function ownNamesAndKeys(value) {
          return {
            names: Object.getOwnPropertyNames(value),
            keys: Reflect.ownKeys(value).map(keyName)
          };
        }
        function constructorSurfaces(globalObject, prototype) {
          var result = {}, names = Object.getOwnPropertyNames(globalObject);
          for (var index = 0; index < names.length; index += 1) {
            var name = names[index];
            var descriptor = Object.getOwnPropertyDescriptor(globalObject, name);
            if (!descriptor || !("value" in descriptor) || typeof descriptor.value !== "function") continue;
            var value = prototype ? descriptor.value.prototype : descriptor.value;
            if (value && (typeof value === "object" || typeof value === "function")) {
              result[name] = ownNamesAndKeys(value);
            }
          }
          return result;
        }
        function objectSurfaces(globalObject) {
          var result = {}, names = Object.getOwnPropertyNames(globalObject);
          for (var index = 0; index < names.length; index += 1) {
            var name = names[index];
            var descriptor = Object.getOwnPropertyDescriptor(globalObject, name);
            if (!descriptor || !("value" in descriptor)) continue;
            var value = descriptor.value;
            if (!value || typeof value !== "object" || value === globalObject) continue;
            if (Object.getOwnPropertyNames(value).length) result[name] = ownNamesAndKeys(value);
          }
          return result;
        }
        self.postMessage({
          global: ownNamesAndKeys(self),
          navigatorPrototype: ownNamesAndKeys(WorkerNavigator.prototype),
          constructorPrototypes: constructorSurfaces(self, true),
          constructorStatics: constructorSurfaces(self, false),
          globalObjects: objectSurfaces(self),
          navigatorChain: (function () {
            var levels = [], current = navigator;
            while (current !== null && levels.length < 8) {
              levels.push(ownNamesAndKeys(current));
              current = Object.getPrototypeOf(current);
            }
            return levels;
          })()
        });
      })();
    `;
  }

  function collectWorker() {
    return new Promise(function (resolve) {
      var blob = new Blob([workerSource()], { type: "text/javascript" });
      var url = URL.createObjectURL(blob);
      var worker = new Worker(url);
      var finish = function (value) {
        worker.terminate();
        URL.revokeObjectURL(url);
        resolve(value);
      };
      worker.onmessage = function (event) { finish(event.data); };
      worker.onerror = function (event) { finish({ error: event.message || "worker error" }); };
    });
  }

  var topBeforeFrame = ownSurface(window);
  var iframe = document.createElement("iframe");
  document.body.appendChild(iframe);
  var childWindow = iframe.contentWindow;
  var result = {
    userAgent: navigator.userAgent,
    topWindowBeforeFrame: topBeforeFrame,
    topWindowWithFrame: ownSurface(window),
    iframeWindow: ownSurface(childWindow),
    navigatorPrototype: ownSurface(Navigator.prototype),
    iframeNavigatorPrototype: ownSurface(childWindow.Navigator.prototype),
    constructorPrototypes: constructorPrototypeSurfaces(window),
    iframeConstructorPrototypes: constructorPrototypeSurfaces(childWindow),
    constructorStatics: constructorStaticSurfaces(window),
    iframeConstructorStatics: constructorStaticSurfaces(childWindow),
    globalObjects: globalObjectSurfaces(window),
    iframeGlobalObjects: globalObjectSurfaces(childWindow),
    worker: await collectWorker(),
    runtimeEvidence: await collectRuntimeEvidence(),
  };
  iframe.remove();
  return result;
})()
