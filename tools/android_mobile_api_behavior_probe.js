// Black-box WebIDL behavior evidence for Android-only Chromium APIs.
// The collector executes this file only inside a valid HTTPS page.
(async () => {
  const describe = value => {
    if (value === undefined) return { type: "undefined" };
    if (value === null) return { type: "null" };
    if (typeof value !== "object" && typeof value !== "function") {
      return { type: typeof value, value };
    }
    return {
      type: typeof value,
      tag: Object.prototype.toString.call(value),
      constructor: value && value.constructor && value.constructor.name,
      names: Object.getOwnPropertyNames(value),
    };
  };
  const sync = callback => {
    try {
      return { ok: true, value: describe(callback()) };
    } catch (error) {
      return {
        ok: false,
        error: {
          name: error && error.name,
          message: error && error.message,
          constructor: error && error.constructor && error.constructor.name,
        },
      };
    }
  };
  const asyncCall = async callback => {
    try {
      return { ok: true, value: describe(await callback()) };
    } catch (error) {
      return {
        ok: false,
        error: {
          name: error && error.name,
          message: error && error.message,
          constructor: error && error.constructor && error.constructor.name,
        },
      };
    }
  };
  const descriptor = (object, name) => {
    const value = Object.getOwnPropertyDescriptor(object, name);
    if (!value) return null;
    return {
      enumerable: value.enumerable,
      configurable: value.configurable,
      writable: value.writable,
      value: "value" in value ? describe(value.value) : undefined,
      get: value.get && { name: value.get.name, length: value.get.length },
      set: value.set && { name: value.set.name, length: value.set.length },
    };
  };
  const constructorProbe = (name, cases) => {
    const constructor = globalThis[name];
    if (typeof constructor !== "function") return { exposed: false };
    const result = {
      exposed: true,
      function: {
        name: constructor.name,
        length: constructor.length,
        source: Function.prototype.toString.call(constructor),
      },
      prototype: Object.getOwnPropertyNames(constructor.prototype),
      descriptors: {},
      calls: {},
    };
    for (const name of Reflect.ownKeys(constructor.prototype)) {
      if (typeof name === "string") result.descriptors[name] = descriptor(constructor.prototype, name);
    }
    result.calls.withoutNew = sync(() => constructor());
    for (const [label, callback] of Object.entries(cases || {})) {
      result.calls[label] = sync(callback);
    }
    return result;
  };

  const result = {
    href: location.href,
    origin: location.origin,
    secureContext: isSecureContext,
    userAgent: navigator.userAgent,
    windowOrientation: sync(() => window.orientation),
    windowHandlers: Object.fromEntries([
      "onorientationchange", "ontouchcancel", "ontouchend", "ontouchmove", "ontouchstart",
    ].map(name => [name, descriptor(window, name)])),
    interfaces: {},
    navigator: {
      platform: navigator.platform,
      product: navigator.product,
      vendor: navigator.vendor,
      language: navigator.language,
      languages: Array.from(navigator.languages),
      hardwareConcurrency: navigator.hardwareConcurrency,
      deviceMemory: navigator.deviceMemory,
      maxTouchPoints: navigator.maxTouchPoints,
      pdfViewerEnabled: navigator.pdfViewerEnabled,
      cookieEnabled: navigator.cookieEnabled,
      contacts: sync(() => navigator.contacts),
      modelContext: sync(() => navigator.modelContext),
      prototypeNames: Object.getOwnPropertyNames(Navigator.prototype),
      userAgentData: sync(() => navigator.userAgentData && ({
        brands: navigator.userAgentData.brands,
        mobile: navigator.userAgentData.mobile,
        platform: navigator.userAgentData.platform,
      })),
      userAgentDataJSON: sync(() => navigator.userAgentData && JSON.stringify({
        brands: navigator.userAgentData.brands,
        mobile: navigator.userAgentData.mobile,
        platform: navigator.userAgentData.platform,
      })),
      plugins: sync(() => Array.from(navigator.plugins, plugin => ({
        name: plugin.name,
        filename: plugin.filename,
        description: plugin.description,
        mimeTypes: Array.from(plugin, mime => mime.type),
      }))),
      mimeTypes: sync(() => Array.from(navigator.mimeTypes, mime => mime.type)),
    },
    connection: navigator.connection ? {
      effectiveType: sync(() => navigator.connection.effectiveType),
      rtt: sync(() => navigator.connection.rtt),
      downlink: sync(() => navigator.connection.downlink),
      saveData: sync(() => navigator.connection.saveData),
      type: sync(() => navigator.connection.type),
      downlinkMax: sync(() => navigator.connection.downlinkMax),
      onchange: sync(() => navigator.connection.onchange),
      ontypechange: sync(() => navigator.connection.ontypechange),
    } : null,
    document: {
      modelContext: sync(() => document.modelContext),
      visibilityState: document.visibilityState,
      hidden: document.hidden,
      hasFocus: document.hasFocus(),
    },
    screen: Object.fromEntries([
      "width", "height", "availWidth", "availHeight", "availLeft", "availTop",
      "colorDepth", "pixelDepth", "isExtended",
    ].map(name => [name, sync(() => screen[name])])),
    screenOrientation: screen.orientation ? {
      type: screen.orientation.type,
      angle: screen.orientation.angle,
    } : null,
    viewport: {
      innerWidth, innerHeight, outerWidth, outerHeight, devicePixelRatio,
      screenX, screenY,
      visualViewport: visualViewport ? Object.fromEntries([
        "width", "height", "offsetLeft", "offsetTop", "pageLeft", "pageTop", "scale",
      ].map(name => [name, visualViewport[name]])) : null,
    },
    userActivation: navigator.userActivation ? {
      hasBeenActive: navigator.userActivation.hasBeenActive,
      isActive: navigator.userActivation.isActive,
    } : null,
    virtualKeyboard: navigator.virtualKeyboard ? {
      prototype: Object.getOwnPropertyNames(Object.getPrototypeOf(navigator.virtualKeyboard)),
      overlaysContent: sync(() => navigator.virtualKeyboard.overlaysContent),
      boundingRect: sync(() => navigator.virtualKeyboard.boundingRect && ({
        x: navigator.virtualKeyboard.boundingRect.x,
        y: navigator.virtualKeyboard.boundingRect.y,
        width: navigator.virtualKeyboard.boundingRect.width,
        height: navigator.virtualKeyboard.boundingRect.height,
      })),
      boundingRectValue: navigator.virtualKeyboard.boundingRect ? {
        x: navigator.virtualKeyboard.boundingRect.x,
        y: navigator.virtualKeyboard.boundingRect.y,
        width: navigator.virtualKeyboard.boundingRect.width,
        height: navigator.virtualKeyboard.boundingRect.height,
      } : null,
      ongeometrychange: sync(() => navigator.virtualKeyboard.ongeometrychange),
    } : null,
    devicePosture: navigator.devicePosture ? {
      prototype: Object.getOwnPropertyNames(Object.getPrototypeOf(navigator.devicePosture)),
      type: sync(() => navigator.devicePosture.type),
      onchange: sync(() => navigator.devicePosture.onchange),
    } : null,
    performance: {
      supportedEntryTypes: Array.from(PerformanceObserver.supportedEntryTypes),
      memory: performance.memory ? {
        jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
        totalJSHeapSize: performance.memory.totalJSHeapSize,
        usedJSHeapSize: performance.memory.usedJSHeapSize,
      } : null,
      nowSamples: Array.from({ length: 8 }, () => performance.now()),
      timeOrigin: performance.timeOrigin,
    },
    inputCapture: (() => {
      const input = document.createElement("input");
      return {
        initial: input.capture,
        descriptor: descriptor(HTMLInputElement.prototype, "capture"),
        afterString: sync(() => (input.capture = "user", input.capture)),
        afterNull: sync(() => (input.capture = null, input.capture)),
      };
    })(),
    mediaQueries: Object.fromEntries([
      "(pointer: coarse)", "(pointer: fine)", "(hover: hover)",
      "(any-pointer: coarse)", "(any-hover: hover)",
    ].map(query => [query, { matches: matchMedia(query).matches, media: matchMedia(query).media }])),
  };

  if (navigator.userAgentData) {
    result.navigator.userAgentDataHighEntropy = await asyncCall(() =>
      navigator.userAgentData.getHighEntropyValues([
        "architecture", "bitness", "formFactors", "fullVersionList", "model",
        "platformVersion", "uaFullVersion", "wow64",
      ])
    );
    result.navigator.userAgentDataHighEntropyJSON = await asyncCall(() =>
      navigator.userAgentData.getHighEntropyValues([
        "architecture", "bitness", "formFactors", "fullVersionList", "model",
        "platformVersion", "uaFullVersion", "wow64",
      ]).then(JSON.stringify)
    );
  }

  result.permissions = {};
  if (navigator.permissions) {
    for (const name of [
      "accelerometer", "background-sync", "camera", "clipboard-read",
      "clipboard-write", "geolocation", "gyroscope", "magnetometer",
      "microphone", "midi", "notifications", "payment-handler",
      "persistent-storage", "speaker-selection", "storage-access",
      "top-level-storage-access", "window-management",
    ]) {
      try {
        const status = await navigator.permissions.query({ name });
        result.permissions[name] = {
          ok: true,
          state: status.state,
          onchange: status.onchange === null ? null : typeof status.onchange,
          prototype: Object.getOwnPropertyNames(Object.getPrototypeOf(status)),
        };
      } catch (error) {
        result.permissions[name] = {
          ok: false,
          error: {
            name: error && error.name,
            message: error && error.message,
            constructor: error && error.constructor && error.constructor.name,
          },
        };
      }
    }
  }

  result.mediaDevices = navigator.mediaDevices ? {
    supportedConstraints: navigator.mediaDevices.getSupportedConstraints(),
    enumerateDevices: await asyncCall(() => navigator.mediaDevices.enumerateDevices().then(
      devices => devices.map(device => ({
        kind: device.kind,
        deviceId: device.deviceId,
        groupId: device.groupId,
        label: device.label,
      }))
    )),
    enumerateDevicesValue: await navigator.mediaDevices.enumerateDevices().then(
      devices => devices.map(device => ({
        kind: device.kind,
        deviceId: device.deviceId,
        groupId: device.groupId,
        label: device.label,
      }))
    ),
  } : null;

  result.speechSynthesis = await (async () => {
    if (!globalThis.speechSynthesis) return null;
    let voices = speechSynthesis.getVoices();
    if (!voices.length) {
      await new Promise(resolve => {
        const timer = setTimeout(resolve, 1500);
        speechSynthesis.addEventListener("voiceschanged", () => {
          clearTimeout(timer);
          resolve();
        }, { once: true });
      });
      voices = speechSynthesis.getVoices();
    }
    return voices.map(voice => ({
      voiceURI: voice.voiceURI,
      name: voice.name,
      lang: voice.lang,
      localService: voice.localService,
      default: voice.default,
    }));
  })();

  result.sensorInterfaces = Object.fromEntries([
    "Accelerometer", "GravitySensor", "Gyroscope", "LinearAccelerationSensor",
    "AbsoluteOrientationSensor", "RelativeOrientationSensor",
    "AmbientLightSensor", "Magnetometer",
  ].map(name => [name, typeof globalThis[name] === "function" ? {
    constructorLength: globalThis[name].length,
    prototype: Object.getOwnPropertyNames(globalThis[name].prototype),
  } : null]));

  result.interfaces.BarcodeDetector = constructorProbe("BarcodeDetector", {
    empty: () => new BarcodeDetector(),
    formats: () => new BarcodeDetector({ formats: ["qr_code"] }),
    invalidFormats: () => new BarcodeDetector({ formats: ["not-a-format"] }),
  });
  if (typeof BarcodeDetector === "function") {
    result.interfaces.BarcodeDetector.staticFormats = await asyncCall(() => BarcodeDetector.getSupportedFormats());
    result.interfaces.BarcodeDetector.detectMissing = await asyncCall(() => new BarcodeDetector().detect());
    result.interfaces.BarcodeDetector.detectImageData = await asyncCall(
      () => new BarcodeDetector().detect(new ImageData(1, 1))
    );
  }

  result.interfaces.ContactAddress = constructorProbe("ContactAddress", {
    empty: () => new ContactAddress(),
  });
  result.interfaces.ContactsManager = constructorProbe("ContactsManager", {
    empty: () => new ContactsManager(),
  });
  if (navigator.contacts) {
    result.navigator.contactsProperties = await asyncCall(() => navigator.contacts.getProperties());
    result.navigator.contactsSelectMissing = await asyncCall(() => navigator.contacts.select());
    result.navigator.contactsSelectEmpty = await asyncCall(() => navigator.contacts.select([]));
    result.navigator.contactsSelectName = await asyncCall(() => navigator.contacts.select(["name"]));
  }

  result.interfaces.ContentIndex = constructorProbe("ContentIndex", {
    empty: () => new ContentIndex(),
  });
  if (typeof ContentIndex === "function") {
    result.interfaces.ContentIndex.addIllegal = await asyncCall(() => ContentIndex.prototype.add.call({}));
    result.interfaces.ContentIndex.deleteIllegal = await asyncCall(() => ContentIndex.prototype.delete.call({}, "x"));
    result.interfaces.ContentIndex.getAllIllegal = await asyncCall(() => ContentIndex.prototype.getAll.call({}));
  }

  result.interfaces.NDEFMessage = constructorProbe("NDEFMessage", {
    empty: () => new NDEFMessage(),
    emptyObject: () => new NDEFMessage({}),
    textRecord: () => new NDEFMessage({ records: [{ recordType: "text", data: "hello" }] }),
  });
  result.interfaces.NDEFRecord = constructorProbe("NDEFRecord", {
    empty: () => new NDEFRecord(),
    emptyObject: () => new NDEFRecord({}),
    textRecord: () => new NDEFRecord({ recordType: "text", data: "hello" }),
  });
  result.interfaces.NDEFReader = constructorProbe("NDEFReader", {
    empty: () => new NDEFReader(),
  });
  if (typeof NDEFReader === "function") {
    const reader = new NDEFReader();
    result.interfaces.NDEFReader.scan = await asyncCall(() => reader.scan());
    result.interfaces.NDEFReader.writeMissing = await asyncCall(() => reader.write());
    result.interfaces.NDEFReader.makeReadOnly = await asyncCall(() => reader.makeReadOnly());
  }
  result.interfaces.NDEFReadingEvent = constructorProbe("NDEFReadingEvent", {
    empty: () => new NDEFReadingEvent(),
    typeOnly: () => new NDEFReadingEvent("reading"),
    valid: () => new NDEFReadingEvent("reading", {
      serialNumber: "serial",
      message: { records: [{ recordType: "text", data: "hello" }] },
    }),
  });

  result.interfaces.ModelContext = constructorProbe("ModelContext", {
    empty: () => new ModelContext(),
  });
  if (navigator.modelContext) {
    result.navigator.modelContextGetTools = await asyncCall(() => navigator.modelContext.getTools());
    result.navigator.modelContextExecuteMissing = await asyncCall(() => navigator.modelContext.executeTool());
    result.navigator.modelContextRegisterMissing = sync(() => navigator.modelContext.registerTool());
  }
  result.interfaces.WebMCPEvent = constructorProbe("WebMCPEvent", {
    empty: () => new WebMCPEvent(),
    typeOnly: () => new WebMCPEvent("toolchange"),
    withTool: () => new WebMCPEvent("toolchange", { toolName: "sample" }),
  });
  return result;
})()
