// 优化1: 使用更真实的时间生成逻辑
function generateProportionalTimes(time1, time2) {
    let result = {};

    // 使用基础比例因子，但为每个键添加小幅波动
    let baseScaleFactor = 0.3 + Math.random() * 0.5; // 0.3 到 0.8 之间，避免极端值

    // 定义时间字段的分组，同组内的字段应该有相近的比例
    const timeGroups = {
        initial: ['ypq', 'vfg', 'ujj'],      // 初始加载阶段
        processing: ['jcv', 'kmw', 'vgm'],   // 处理阶段
        completion: ['uux', 'gdt', 'kgm', 'rxr'],  // 完成阶段
        final: ['lmw', 'xwq', 'zcv']         // 最终阶段
    };

    // 为每个组生成略有不同的比例因子
    const groupFactors = {};
    for (let group in timeGroups) {
        // 在基础因子附近添加 ±5% 的波动
        groupFactors[group] = baseScaleFactor * (0.95 + Math.random() * 0.1);
    }

    // 找出每个键属于哪个组
    function getGroupFactor(key) {
        for (let group in timeGroups) {
            if (timeGroups[group]. includes(key)) {
                return groupFactors[group];
            }
        }
        return baseScaleFactor;
    }

    for (let key in time1) {
        if (time2. hasOwnProperty(key)) {
            let min = time1[key];
            let max = time2[key];
            let delta = max - min;

            // 使用组内因子，并添加微小的个体波动 (±2%)
            let individualFactor = getGroupFactor(key) * (0.98 + Math.random() * 0.04);

            // 添加高斯噪声使数据更自然
            let noise = gaussianRandom() * delta * 0.01; // 1% 的高斯噪声

            result[key] = Math.max(min, min + delta * individualFactor + noise);
        }
    }

    // 确保时间顺序的逻辑性
    result = ensureTimeOrder(result);

    return result;
}

// 高斯随机数生成（Box-Muller 变换）
function gaussianRandom(mean = 0, stdev = 1) {
    const u = 1 - Math.random();
    const v = Math.random();
    const z = Math.sqrt(-2.0 * Math. log(u)) * Math.cos(2.0 * Math.PI * v);
    return z * stdev + mean;
}

// 确保时间字段的顺序逻辑
function ensureTimeOrder(times) {
    const orderedKeys = ['ypq', 'vfg', 'ujj', 'jcv', 'kmw', 'vgm', 'uux', 'gdt', 'kgm', 'rxr', 'lmw', 'xwq', 'zcv'];

    let lastTime = 0;
    for (let key of orderedKeys) {
        if (times[key] !== undefined) {
            if (times[key] < lastTime) {
                times[key] = lastTime + Math.random() * 0.5; // 添加小的增量
            }
            lastTime = times[key];
        }
    }
    return times;
}

// 优化2: 更真实的性能数据调整
function adjustPerformanceData(perforFile) {
    let newPerforFile = JSON.parse(JSON. stringify(perforFile));

    for (let key in newPerforFile) {
        let entry = newPerforFile[key];
        if (typeof entry !== "object") continue;

        // 基于真实浏览器行为的时间调整
        let adjustedEntry = adjustEntryTiming(entry);
        newPerforFile[key] = adjustedEntry;
    }

    return newPerforFile;
}

function adjustEntryTiming(entry) {
    let adjusted = { ...entry };

    // 网络延迟的真实范围 (毫秒)
    const networkJitter = gaussianRandom(0, 3); // 网络抖动
    const serverProcessingVariance = gaussianRandom(0, 5); // 服务器处理时间波动

    // DNS 查询时间 (如果需要查询的话)
    if (adjusted.domainLookupEnd > adjusted.domainLookupStart) {
        let dnsTime = adjusted.domainLookupEnd - adjusted. domainLookupStart;
        dnsTime = Math. max(0, dnsTime + gaussianRandom(0, dnsTime * 0.1));
        adjusted.domainLookupEnd = adjusted.domainLookupStart + dnsTime;
    }

    // 连接时间
    if (adjusted.connectEnd > adjusted.connectStart) {
        let connectTime = adjusted. connectEnd - adjusted. connectStart;
        connectTime = Math. max(0, connectTime + gaussianRandom(0, connectTime * 0.05));
        adjusted.connectEnd = adjusted. connectStart + connectTime;
    }

    // 请求到响应的时间 (TTFB - Time To First Byte)
    if (adjusted.responseStart && adjusted.requestStart) {
        let ttfb = adjusted.responseStart - adjusted.requestStart;
        // TTFB 波动通常在 5-15% 范围内
        ttfb = Math. max(1, ttfb + gaussianRandom(0, ttfb * 0.1) + serverProcessingVariance);
        adjusted.responseStart = adjusted.requestStart + ttfb;
    }

    // 响应下载时间
    if (adjusted.responseEnd && adjusted.responseStart) {
        let downloadTime = adjusted.responseEnd - adjusted.responseStart;
        // 下载时间波动与内容大小相关
        let sizeBasedVariance = (adjusted.encodedBodySize || 1000) / 100000; // 基于大小的波动
        downloadTime = Math.max(0.1, downloadTime + gaussianRandom(0, downloadTime * 0.05) + networkJitter);
        adjusted.responseEnd = adjusted.responseStart + downloadTime;
    }

    // 确保时间顺序正确
    adjusted = ensurePerformanceTimeOrder(adjusted);

    // 重新计算 duration
    if (adjusted.startTime !== undefined && adjusted.responseEnd !== undefined) {
        adjusted.duration = Math.max(0, adjusted.responseEnd - adjusted.startTime);
    }

    // 添加真实的小数位数 (浏览器通常保留到微秒级别)
    for (let key in adjusted) {
        if (typeof adjusted[key] === 'number' && key !== 'transferSize' &&
            key !== 'encodedBodySize' && key !== 'decodedBodySize' &&
            key !== 'responseStatus' && key !== 'redirectCount') {
            // 保留更真实的精度 (通常是 5-6 位小数)
            adjusted[key] = Math.round(adjusted[key] * 100000) / 100000;
        }
    }

    return adjusted;
}

function ensurePerformanceTimeOrder(entry) {
    const timeOrder = [
        'startTime',
        'redirectStart',
        'redirectEnd',
        'fetchStart',
        'domainLookupStart',
        'domainLookupEnd',
        'connectStart',
        'secureConnectionStart',
        'connectEnd',
        'requestStart',
        'responseStart',
        'firstInterimResponseStart',
        'finalResponseHeadersStart',
        'responseEnd'
    ];

    let lastNonZeroTime = 0;

    for (let key of timeOrder) {
        if (entry[key] !== undefined && entry[key] > 0) {
            if (entry[key] < lastNonZeroTime) {
                entry[key] = lastNonZeroTime + Math.random() * 0.1;
            }
            lastNonZeroTime = entry[key];
        }
    }

    return entry;
}

// 优化3: 添加更真实的浏览器指纹特征
function addRealisticBrowserCharacteristics(perforFile) {
    let newPerforFile = JSON.parse(JSON.stringify(perforFile));

    for (let key in newPerforFile) {
        let entry = newPerforFile[key];
        if (typeof entry !== "object") continue;

        // 真实浏览器的 transferSize 通常略大于 encodedBodySize (HTTP 头部)
        if (entry. encodedBodySize && entry.transferSize) {
            let headerSize = 200 + Math.floor(Math.random() * 300); // HTTP 头部大小 200-500 bytes
            entry.transferSize = entry.encodedBodySize + headerSize;
        }

        // 某些时间字段在特定条件下应该为 0
        if (entry.redirectCount === 0) {
            entry.redirectStart = 0;
            entry.redirectEnd = 0;
        }

        if (entry.workerStart === 0) {
            // 如果没有 Service Worker，保持为 0
        }

        newPerforFile[key] = entry;
    }

    return newPerforFile;
}
time1 = {
    "ypq": 1.01,
    "vfg": 561.01,
    "ujj": 1.02,
    "jcv": 1.03,
    "kmw": 1.06,
    "vgm": 1.07,
    "uux": 1.08,
    "gdt": 1.09,
    "lmw": 7.059999999999893,
    "xwq": 7.069999999999893,
    "zcv": 7.079999999999893,
    "kgm": 1.08,
    "rxr": 1.09
}
time2 = {
    "ypq": 1348.0999999996275,
    "vfg": 1348.0999999996275,
    "ujj": 1348.3999999994412,
    "jcv": 1352.7999999998137,
    "kmw": 1418.2000000001863,
    "vgm": 1432.3999999994412,
    "uux": 1437.2999999998137,
    "gdt": 1446.8999999994412,
    "lmw": 1464,
    "xwq": 1464,
    "zcv": 1467.3999999994412,
    "kgm": 1437.2999999998137,
    "rxr": 1446.8999999994412
}
perfor_file = {
    "eux": {
        "name": "https://arcteryx.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/fp?x-kpsdk-v=j-1.0.0",
        "entryType": "navigation",
        "startTime": 0,
        "duration": 0,
        "initiatorType": "navigation",
        "deliveryType": "",
        "nextHopProtocol": "h2",
        "renderBlockingStatus": "non-blocking",
        "workerStart": 0,
        "redirectStart": 0,
        "redirectEnd": 0,
        "fetchStart": 2.8999999999068677,
        "domainLookupStart": 2.8999999999068677,
        "domainLookupEnd": 2.8999999999068677,
        "connectStart": 2.8999999999068677,
        "secureConnectionStart": 2.8999999999068677,
        "connectEnd": 2.8999999999068677,
        "requestStart": 8.699999999720603,
        "responseStart": 323.79999999981374,
        "firstInterimResponseStart": 0,
        "finalResponseHeadersStart": 323.79999999981374,
        "responseEnd": 324.89999999990687,
        "transferSize": 851,
        "encodedBodySize": 551,
        "decodedBodySize": 762,
        "responseStatus": 429,
        "serverTiming": [],
        "unloadEventStart": 0,
        "unloadEventEnd": 0,
        "domInteractive": 1586.1999999997206,
        "domContentLoadedEventStart": 0,
        "domContentLoadedEventEnd": 0,
        "domComplete": 0,
        "loadEventStart": 0,
        "loadEventEnd": 0,
        "type": "navigate",
        "redirectCount": 0,
        "activationStart": 0,
        "criticalCHRestart": 0,
        "notRestoredReasons": null
    },
    "nzv": {
        "name": "https://arcteryx.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/ips.js?KP_UIDz=0HuxsXYPBXS5ptrylOYopLKA44YVo3FTIqQX6nOGBp6QdQrDlgJMi2SH7XowsF8VPftA8mW10SEYTnn6K0b5FexuLSaK4qE7WG7ZZSQm6PY3iItR52I2gdkBUnyyIGwBtrSdRKLTyAzZZs1ZBveAeCBoCygnX32LnjvXcItCsh2K&x-kpsdk-v=j-1.0.0&x-kpsdk-im=CiRjNzkzMzJiYS02OTMxLTQxY2ItYjAwZS0yODQzNDExOTY0YzI",
        "entryType": "resource",
        "startTime": 400.60000000009313,
        "duration": 1076.2999999998137,
        "initiatorType": "script",
        "deliveryType": "",
        "nextHopProtocol": "h2",
        "renderBlockingStatus": "non-blocking",
        "workerStart": 0,
        "redirectStart": 0,
        "redirectEnd": 0,
        "fetchStart": 400.60000000009313,
        "domainLookupStart": 400.60000000009313,
        "domainLookupEnd": 400.60000000009313,
        "connectStart": 400.60000000009313,
        "secureConnectionStart": 400.60000000009313,
        "connectEnd": 400.60000000009313,
        "requestStart": 406.29999999981374,
        "responseStart": 1473.7999999998137,
        "firstInterimResponseStart": 0,
        "finalResponseHeadersStart": 1473.7999999998137,
        "responseEnd": 1476.8999999999069,
        "transferSize": 186758,
        "encodedBodySize": 186458,
        "decodedBodySize": 1802171,
        "responseStatus": 200,
        "serverTiming": []
    }
}

// 主执行流程
let randomTimes = generateProportionalTimes(time1, time2);
perfor_file = adjustPerformanceData(perfor_file);
perfor_file = addRealisticBrowserCharacteristics(perfor_file);

window = global;

reg_4 = function (p1, p2) {
    let thisProp = this, reg_3 = arguments, reg_4 = p1, reg_5 = p2, memory3039_scope4, memory3040_scope4,
        memory3041_scope4, memory3042_scope4, memory3043_scope4, memory3044_scope4, memory3045_scope4,
        memory3046_scope4, memory3047_scope4, memory3048_scope4, memory3049_scope4, memory3050_scope4,
        memory3051_scope4, memory3052_scope4, memory3053_scope4, memory3054_scope4, reg_7, reg_6, reg_9, reg_8, reg_10,
        reg_11, reg_12, reg_13, retVal, reg_14, reg_15, reg_16;
    memory3039_scope4 = reg_4;
    memory3040_scope4 = reg_5;
    reg_4 = window.Math;
    reg_7 = reg_4.floor;
    memory3041_scope4 = reg_7;
    reg_4 = window.Math;
    reg_5 = reg_4.round;
    memory3042_scope4 = reg_5;
    reg_4 = window.Math;
    reg_5 = reg_4.random;
    memory3043_scope4 = reg_5;
    memory3044_scope4 = 1;
    memory3045_scope4 = 32;
    reg_4 = new Array(4);
    reg_4[0] = "w";
    reg_4[1] = "x";
    reg_4[2] = "y";
    reg_4[3] = "z";
    memory3046_scope4 = reg_4;
    reg_6 = new Array(8);
    reg_4 = new Array(2);
    reg_4[0] = 0;
    reg_7 = memory3039_scope4;
    reg_5 = reg_7.zcv;
    reg_7 = memory3039_scope4;
    reg_9 = reg_7.ypq;
    reg_8 = reg_5 - reg_9;
    reg_4[1] = reg_8;
    reg_6[0] = reg_4;
    reg_4 = new Array(2);
    reg_4[0] = 1;
    reg_5 = memory3039_scope4;
    reg_8 = reg_5.vfg;
    reg_5 = memory3039_scope4;
    reg_7 = reg_5.ypq;
    reg_5 = reg_8 - reg_7;
    reg_4[1] = reg_5;
    reg_6[1] = reg_4;
    reg_8 = new Array(2);
    reg_8[0] = 2;
    reg_5 = memory3039_scope4;
    reg_9 = reg_5.ujj;
    reg_5 = memory3039_scope4;
    reg_4 = reg_5.vfg;
    reg_5 = reg_9 - reg_4;
    reg_8[1] = reg_5;
    reg_6[2] = reg_8;
    reg_4 = new Array(2);
    reg_4[0] = 3;
    reg_5 = memory3039_scope4;
    reg_7 = reg_5.jcv;
    reg_8 = memory3039_scope4;
    reg_5 = reg_8.ujj;
    reg_8 = reg_7 - reg_5;
    reg_4[1] = reg_8;
    reg_6[3] = reg_4;
    reg_4 = new Array(2);
    reg_4[0] = 4;
    reg_7 = memory3039_scope4;
    reg_5 = reg_7.kmw;
    reg_7 = memory3039_scope4;
    reg_8 = reg_7.jcv;
    reg_7 = reg_5 - reg_8;
    reg_4[1] = reg_7;
    reg_6[4] = reg_4;
    reg_4 = new Array(2);
    reg_4[0] = 5;
    reg_5 = memory3039_scope4;
    reg_7 = reg_5.rxr;
    reg_8 = memory3039_scope4;
    reg_10 = reg_8.kmw;
    reg_5 = reg_7 - reg_10;
    reg_4[1] = reg_5;
    reg_6[5] = reg_4;
    reg_4 = new Array(2);
    reg_4[0] = 6;
    reg_5 = memory3039_scope4;
    reg_8 = reg_5.lmw;
    reg_5 = memory3039_scope4;
    reg_7 = reg_5.rxr;
    reg_5 = reg_8 - reg_7;
    reg_4[1] = reg_5;
    reg_6[6] = reg_4;
    reg_4 = new Array(2);
    reg_4[0] = 7;
    reg_5 = memory3039_scope4;
    reg_7 = reg_5.zcv;
    reg_5 = memory3039_scope4;
    reg_9 = reg_5.xwq;
    reg_5 = reg_7 - reg_9;
    reg_4[1] = reg_5;
    reg_6[7] = reg_4;
    memory3047_scope4 = reg_6;
    reg_4 = memory3040_scope4;
    reg_5 = reg_4 !== null;
    if (reg_5) {
        reg_6 = memory3040_scope4;
        reg_4 = undefined;
        reg_7 = reg_6 !== reg_4;
        reg_5 = reg_7;
    }
    if (reg_5) {
        reg_6 = memory3040_scope4;
        reg_4 = reg_6.eux;
        reg_5 = reg_4;
    }
    if (reg_5) {
        reg_4 = memory3040_scope4;
        reg_5 = reg_4.eux;
        memory3048_scope4 = reg_5;
        reg_5 = memory3047_scope4;
        reg_6 = reg_5.push;
        reg_7 = new Array(3);
        reg_8 = new Array(2);
        reg_8[0] = 8;
        reg_10 = memory3048_scope4;
        reg_9 = reg_10.requestStart;
        reg_10 = memory3048_scope4;
        reg_11 = reg_10.fetchStart;
        reg_10 = reg_9 - reg_11;
        reg_8[1] = reg_10;
        reg_7[0] = reg_8;
        reg_8 = new Array(2);
        reg_8[0] = 9;
        reg_9 = memory3048_scope4;
        reg_11 = reg_9.responseStart;
        reg_9 = memory3048_scope4;
        reg_12 = reg_9.requestStart;
        reg_9 = reg_11 - reg_12;
        reg_8[1] = reg_9;
        reg_7[1] = reg_8;
        reg_9 = new Array(2);
        reg_9[0] = 10;
        reg_8 = memory3048_scope4;
        reg_10 = reg_8.responseEnd;
        reg_8 = memory3048_scope4;
        reg_11 = reg_8.responseStart;
        reg_13 = reg_10 - reg_11;
        reg_9[1] = reg_13;
        reg_7[2] = reg_9;
        retVal = reg_6.apply(reg_5, reg_7);
        reg_4 = retVal;
        reg_4 = memory3047_scope4;
        reg_6 = reg_4[0];
        reg_4 = reg_6[1];
        reg_7 = memory3048_scope4;
        reg_10 = reg_7.responseEnd;
        reg_7 = memory3048_scope4;
        reg_8 = reg_7.fetchStart;
        reg_7 = reg_10 - reg_8;
        reg_7 = reg_4 + reg_7;
        reg_6[1] = reg_7;
    }
    reg_4 = memory3040_scope4;
    reg_7 = reg_4 !== null;
    if (reg_7) {
        reg_4 = memory3040_scope4;
        reg_6 = undefined;
        reg_8 = reg_4 !== reg_6;
        reg_7 = reg_8;
    }
    if (reg_7) {
        reg_4 = memory3040_scope4;
        reg_6 = reg_4.nzv;
        reg_7 = reg_6;
    }
    if (reg_7) {
        reg_7 = memory3040_scope4;
        reg_4 = reg_7.nzv;
        memory3049_scope4 = reg_4;
        reg_4 = memory3047_scope4;
        reg_8 = reg_4.push;
        reg_7 = new Array(3);
        reg_9 = new Array(2);
        reg_9[0] = 11;
        reg_10 = memory3049_scope4;
        reg_11 = reg_10.requestStart;
        reg_10 = memory3049_scope4;
        reg_13 = reg_10.fetchStart;
        reg_10 = reg_11 - reg_13;
        reg_9[1] = reg_10;
        reg_7[0] = reg_9;
        reg_10 = new Array(2);
        reg_10[0] = 12;
        reg_9 = memory3049_scope4;
        reg_11 = reg_9.responseStart;
        reg_13 = memory3049_scope4;
        reg_9 = reg_13.requestStart;
        reg_14 = reg_11 - reg_9;
        reg_10[1] = reg_14;
        reg_7[1] = reg_10;
        reg_9 = new Array(2);
        reg_9[0] = 13;
        reg_10 = memory3049_scope4;
        reg_11 = reg_10.responseEnd;
        reg_10 = memory3049_scope4;
        reg_12 = reg_10.responseStart;
        reg_10 = reg_11 - reg_12;
        reg_9[1] = reg_10;
        reg_7[2] = reg_9;
        retVal = reg_8.apply(reg_4, reg_7);
        reg_6 = retVal;
    }
    reg_8 = memory3044_scope4;
    reg_10 = reg_8.toString;
    reg_7 = new Array(1);
    reg_9 = memory3045_scope4;
    reg_7[0] = reg_9;
    retVal = reg_10.apply(reg_8, reg_7);
    reg_6 = retVal;
    memory3050_scope4 = reg_6;
    reg_7 = !1;
    memory3051_scope4 = reg_7;
    reg_6 = memory3047_scope4;
    reg_7 = reg_6.length;
    memory3052_scope4 = reg_7;
    reg_9 = memory3052_scope4;
    reg_6 = reg_9 > 0;
    while (reg_6) {
        reg_7 = memory3041_scope4;
        reg_12 = memory3043_scope4;
        reg_10 = reg_12();
        reg_11 = memory3052_scope4;
        reg_9 = reg_10 * reg_11;
        reg_6 = reg_7(reg_9);
        memory3053_scope4 = reg_6;
        reg_7 = memory3042_scope4;
        reg_9 = memory3047_scope4;
        reg_11 = memory3053_scope4;
        reg_10 = reg_9[reg_11];
        reg_9 = reg_10[1];
        reg_6 = reg_7(reg_9);
        memory3054_scope4 = reg_6;
        reg_7 = window.isFinite;
        reg_9 = memory3054_scope4;
        reg_6 = reg_7(reg_9);
        if (reg_6) {
            reg_9 = memory3051_scope4;
            if (reg_9) {
                reg_9 = memory3050_scope4;
                reg_10 = memory3046_scope4;
                reg_14 = memory3041_scope4;
                reg_15 = memory3043_scope4;
                reg_13 = reg_15();
                reg_15 = memory3046_scope4;
                reg_16 = reg_15.length;
                reg_15 = reg_13 * reg_16;
                reg_12 = reg_14(reg_15);
                reg_11 = reg_10[reg_12];
                reg_11 = reg_9 + reg_11;
                memory3050_scope4 = reg_11;
                reg_7 = reg_11;
            } else {
                reg_10 = !0;
                memory3051_scope4 = reg_10;
                reg_7 = reg_10;
            }
            reg_7 = memory3050_scope4;
            reg_12 = memory3047_scope4;
            reg_13 = memory3053_scope4;
            reg_10 = reg_12[reg_13];
            reg_9 = reg_10[0];
            reg_10 = reg_9.toString;
            reg_14 = new Array(1);
            reg_13 = memory3045_scope4;
            reg_14[0] = reg_13;
            retVal = reg_10.apply(reg_9, reg_14);
            reg_11 = retVal;
            reg_12 = memory3054_scope4;
            reg_15 = reg_12.toString;
            reg_13 = new Array(1);
            reg_14 = memory3045_scope4;
            reg_13[0] = reg_14;
            retVal = reg_15.apply(reg_12, reg_13);
            reg_10 = retVal;
            reg_13 = reg_11 + reg_10;
            reg_13 = reg_7 + reg_13;
            memory3050_scope4 = reg_13;
            reg_6 = reg_13;
        }
        reg_7 = memory3047_scope4;
        reg_11 = reg_7.splice;
        reg_10 = new Array(2);
        reg_14 = memory3053_scope4;
        reg_10[0] = reg_14;
        reg_10[1] = 1;
        retVal = reg_11.apply(reg_7, reg_10);
        reg_6 = retVal;
        reg_6 = memory3052_scope4;
        reg_10 = 1;
        reg_10 = reg_6 - reg_10;
        memory3052_scope4 = reg_10;
        reg_9 = memory3052_scope4;
        reg_6 = reg_9 > 0;
    }
    reg_6 = memory3050_scope4;
    return reg_6;
};

function get_dt() {
    return reg_4(randomTimes, perfor_file);
}

console.log(get_dt())