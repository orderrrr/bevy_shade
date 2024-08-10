let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error(`expected a boolean argument, found ${typeof(n)}`);
    }
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (typeof(arg) !== 'string') throw new Error(`expected a string argument, found ${typeof(arg)}`);

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error(`expected a number argument, found ${typeof(n)}`);
}
function __wbg_adapter_26(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5f79bf1caa58a081(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_29(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h71756065329dd073(arg0, arg1, addHeapObject(arg2));
}

/**
* @param {number} width
* @param {number} height
* @param {string} bind_id
* @returns {Promise<WgpuToyRenderer>}
*/
export function create_renderer(width, height, bind_id) {
    _assertNum(width);
    _assertNum(height);
    const ptr0 = passStringToWasm0(bind_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.create_renderer(width, height, ptr0, len0);
    return takeObject(ret);
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getUint32Memory0();
    for (let i = 0; i < array.length; i++) {
        mem[ptr / 4 + i] = addHeapObject(array[i]);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

let cachedFloat32Memory0 = null;

function getFloat32Memory0() {
    if (cachedFloat32Memory0 === null || cachedFloat32Memory0.byteLength === 0) {
        cachedFloat32Memory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32Memory0;
}

function passArrayF32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getFloat32Memory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
function __wbg_adapter_423(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h5e291eb4dc40e0fa(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

const SourceMapFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_sourcemap_free(ptr >>> 0));
/**
*/
export class SourceMap {

    constructor() {
        throw new Error('cannot invoke `new` directly');
    }

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SourceMap.prototype);
        obj.__wbg_ptr = ptr;
        SourceMapFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SourceMapFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_sourcemap_free(ptr);
    }
}

const WgpuToyRendererFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wgputoyrenderer_free(ptr >>> 0));
/**
*/
export class WgpuToyRenderer {

    constructor() {
        throw new Error('cannot invoke `new` directly');
    }

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WgpuToyRenderer.prototype);
        obj.__wbg_ptr = ptr;
        WgpuToyRendererFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WgpuToyRendererFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wgputoyrenderer_free(ptr);
    }
    /**
    */
    render() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wgputoyrenderer_render(this.__wbg_ptr);
    }
    /**
    * @returns {string}
    */
    prelude() {
        let deferred1_0;
        let deferred1_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertNum(this.__wbg_ptr);
            wasm.wgputoyrenderer_prelude(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @param {string} shader
    * @returns {Promise<any>}
    */
    preprocess(shader) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(shader, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wgputoyrenderer_preprocess(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
    * @param {SourceMap} source
    */
    compile(source) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertClass(source, SourceMap);
        if (source.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        var ptr0 = source.__destroy_into_raw();
        wasm.wgputoyrenderer_compile(this.__wbg_ptr, ptr0);
    }
    /**
    * @param {number} t
    */
    set_time_elapsed(t) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wgputoyrenderer_set_time_elapsed(this.__wbg_ptr, t);
    }
    /**
    * @param {number} t
    */
    set_time_delta(t) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wgputoyrenderer_set_time_delta(this.__wbg_ptr, t);
    }
    /**
    * @param {number} x
    * @param {number} y
    */
    set_mouse_pos(x, y) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wgputoyrenderer_set_mouse_pos(this.__wbg_ptr, x, y);
    }
    /**
    * @param {boolean} click
    */
    set_mouse_click(click) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBoolean(click);
        wasm.wgputoyrenderer_set_mouse_click(this.__wbg_ptr, click);
    }
    /**
    * @param {number} keycode
    * @param {boolean} keydown
    */
    set_keydown(keycode, keydown) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(keycode);
        _assertBoolean(keydown);
        wasm.wgputoyrenderer_set_keydown(this.__wbg_ptr, keycode, keydown);
    }
    /**
    * @param {(string)[]} names
    * @param {Float32Array} values
    */
    set_custom_floats(names, values) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArrayJsValueToWasm0(names, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF32ToWasm0(values, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        wasm.wgputoyrenderer_set_custom_floats(this.__wbg_ptr, ptr0, len0, ptr1, len1);
    }
    /**
    * @param {boolean} pass_f32
    */
    set_pass_f32(pass_f32) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBoolean(pass_f32);
        wasm.wgputoyrenderer_set_pass_f32(this.__wbg_ptr, pass_f32);
    }
    /**
    * @param {number} width
    * @param {number} height
    * @param {number} scale
    */
    resize(width, height, scale) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(width);
        _assertNum(height);
        wasm.wgputoyrenderer_resize(this.__wbg_ptr, width, height, scale);
    }
    /**
    */
    reset() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wgputoyrenderer_reset(this.__wbg_ptr);
    }
    /**
    * @param {Function} callback
    */
    on_success(callback) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wgputoyrenderer_on_success(this.__wbg_ptr, addHeapObject(callback));
    }
    /**
    * @param {number} index
    * @param {Uint8Array} bytes
    */
    load_channel(index, bytes) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(index);
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wgputoyrenderer_load_channel(this.__wbg_ptr, index, ptr0, len0);
    }
    /**
    * @param {number} index
    * @param {Uint8Array} bytes
    */
    load_channel_hdr(index, bytes) {
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertNum(this.__wbg_ptr);
            _assertNum(index);
            const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.wgputoyrenderer_load_channel_hdr(retptr, this.__wbg_ptr, index, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

export function __wbg_wgslerrorhandler_eb3a85e0b82b6f2e() { return logError(function (arg0, arg1, arg2, arg3) {
    wgsl_error_handler(getStringFromWasm0(arg0, arg1), arg2 >>> 0, arg3 >>> 0);
}, arguments) };

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    _assertBoolean(ret);
    return ret;
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export function __wbg_sourcemap_new() { return logError(function (arg0) {
    const ret = SourceMap.__wrap(arg0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_wgputoyrenderer_new() { return logError(function (arg0) {
    const ret = WgpuToyRenderer.__wrap(arg0);
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_string_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_cb_drop(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    _assertBoolean(ret);
    return ret;
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbg_error_f851667af71bcfc6() { return logError(function (arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

export function __wbg_new_abda76e883ba8a5f() { return logError(function () {
    const ret = new Error();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_stack_658279fe44541cf6() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_Window_94d759f1f207a15b() { return logError(function (arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_WorkerGlobalScope_b13c8cef62388de9() { return logError(function (arg0) {
    const ret = getObject(arg0).WorkerGlobalScope;
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};

export function __wbindgen_is_object(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    _assertBoolean(ret);
    return ret;
};

export function __wbg_gpu_1f3675e2d4aa88f4() { return logError(function (arg0) {
    const ret = getObject(arg0).gpu;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_dispatchWorkgroups_4bc133944e89d5e0() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
}, arguments) };

export function __wbg_dispatchWorkgroupsIndirect_8050acb60dd74a34() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
}, arguments) };

export function __wbg_end_28d311f5d435aa6d() { return logError(function (arg0) {
    getObject(arg0).end();
}, arguments) };

export function __wbg_setPipeline_8630b264a9c4ec4b() { return logError(function (arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
}, arguments) };

export function __wbg_setBindGroup_17e73587d3c1be08() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
}, arguments) };

export function __wbg_setBindGroup_5a450a0e97199c15() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
}, arguments) };

export function __wbg_message_e73620d927b54373() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_has_1509b2ce6759dc2a() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).has(getStringFromWasm0(arg1, arg2));
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_error_c4453561fa6c2209() { return logError(function (arg0) {
    const ret = getObject(arg0).error;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_size_fc880d60ff425a47() { return logError(function (arg0) {
    const ret = getObject(arg0).size;
    return ret;
}, arguments) };

export function __wbg_usage_5e9a3548afbc3ebb() { return logError(function (arg0) {
    const ret = getObject(arg0).usage;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_destroy_199808599201ee27() { return logError(function (arg0) {
    getObject(arg0).destroy();
}, arguments) };

export function __wbg_getMappedRange_1216b00d6d7803de() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getMappedRange(arg1, arg2);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_mapAsync_3b0a03a892fb22b3() { return logError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_unmap_7a0dddee82ac6ed3() { return logError(function (arg0) {
    getObject(arg0).unmap();
}, arguments) };

export function __wbg_getBindGroupLayout_a0d36a72bd39bb04() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_copyExternalImageToTexture_87bdcc3260c6efba() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };

export function __wbg_submit_afbd82b0d5056194() { return logError(function (arg0, arg1) {
    getObject(arg0).submit(getObject(arg1));
}, arguments) };

export function __wbg_writeBuffer_4245ce84e6d772c9() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
}, arguments) };

export function __wbg_writeTexture_686a8160c3c5ddbb() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
}, arguments) };

export function __wbg_label_175c4f59b3eca611() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).label;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_beginComputePass_a148b983810f6795() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).beginComputePass(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_beginRenderPass_0b83360fd99b5810() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).beginRenderPass(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_clearBuffer_2cc723ab6b818737() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2);
}, arguments) };

export function __wbg_clearBuffer_78a94a2eda97eb5a() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
}, arguments) };

export function __wbg_copyBufferToBuffer_667953bc6dccb6b4() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
}, arguments) };

export function __wbg_copyBufferToTexture_ca5b298687bed60a() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };

export function __wbg_copyTextureToBuffer_cdf8118386295eb4() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };

export function __wbg_copyTextureToTexture_67678f03fd20bd23() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
}, arguments) };

export function __wbg_finish_ce7d5c15fce975aa() { return logError(function (arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_finish_d1d9eb9915c96a79() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_resolveQuerySet_22e31015a36a09d5() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
}, arguments) };

export function __wbg_instanceof_GpuDeviceLostInfo_22f963b61044b3b1() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUDeviceLostInfo;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_reason_3af8e4afbe0efdd8() { return logError(function (arg0) {
    const ret = getObject(arg0).reason;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_message_3bef8c43f84eab9c() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_instanceof_GpuOutOfMemoryError_3621d9e8ec05691e() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUOutOfMemoryError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_getBindGroupLayout_abc654a192f85d5e() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_finish_2115db9e679c5aae() { return logError(function (arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_finish_4a754149a60eddc0() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_setBindGroup_58e27d4cd266f187() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
}, arguments) };

export function __wbg_setBindGroup_f70bb0d0a5ace56d() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
}, arguments) };

export function __wbg_draw_60508d893ce4e012() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
}, arguments) };

export function __wbg_drawIndexed_d5c5dff02437a4f0() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
}, arguments) };

export function __wbg_drawIndexedIndirect_bf668464170261b3() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
}, arguments) };

export function __wbg_drawIndirect_54f93ae4ccc85358() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
}, arguments) };

export function __wbg_setIndexBuffer_747e1ba3f58d7227() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
}, arguments) };

export function __wbg_setIndexBuffer_3f1635c89f72d661() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
}, arguments) };

export function __wbg_setPipeline_a95b89d99620ba34() { return logError(function (arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
}, arguments) };

export function __wbg_setVertexBuffer_94a88edbfb4b07f8() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
}, arguments) };

export function __wbg_setVertexBuffer_407067a9522118df() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
}, arguments) };

export function __wbg_maxTextureDimension1D_ea59b0f0cc2e29cd() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureDimension1D;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxTextureDimension2D_00984ba245729ced() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureDimension2D;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxTextureDimension3D_95c3d3adb6d66ec5() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureDimension3D;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxTextureArrayLayers_68f4a1218a54fa93() { return logError(function (arg0) {
    const ret = getObject(arg0).maxTextureArrayLayers;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxBindGroups_e76fb8650a4459d7() { return logError(function (arg0) {
    const ret = getObject(arg0).maxBindGroups;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxBindingsPerBindGroup_2af20f39aef3fd86() { return logError(function (arg0) {
    const ret = getObject(arg0).maxBindingsPerBindGroup;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxDynamicUniformBuffersPerPipelineLayout_074c891075b375b7() { return logError(function (arg0) {
    const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxDynamicStorageBuffersPerPipelineLayout_b91e3e6efb7b7a8c() { return logError(function (arg0) {
    const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxSampledTexturesPerShaderStage_76354979d03a2b27() { return logError(function (arg0) {
    const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxSamplersPerShaderStage_fe8d223de90e5459() { return logError(function (arg0) {
    const ret = getObject(arg0).maxSamplersPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxStorageBuffersPerShaderStage_bced69629145d26d() { return logError(function (arg0) {
    const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxStorageTexturesPerShaderStage_fcf51f22620c0092() { return logError(function (arg0) {
    const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxUniformBuffersPerShaderStage_b3b013238400f0c0() { return logError(function (arg0) {
    const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxUniformBufferBindingSize_194fd7147cf2e95a() { return logError(function (arg0) {
    const ret = getObject(arg0).maxUniformBufferBindingSize;
    return ret;
}, arguments) };

export function __wbg_maxStorageBufferBindingSize_78504383af63ac53() { return logError(function (arg0) {
    const ret = getObject(arg0).maxStorageBufferBindingSize;
    return ret;
}, arguments) };

export function __wbg_minUniformBufferOffsetAlignment_4880e6786cb7ec5d() { return logError(function (arg0) {
    const ret = getObject(arg0).minUniformBufferOffsetAlignment;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_minStorageBufferOffsetAlignment_9913f200aee2c749() { return logError(function (arg0) {
    const ret = getObject(arg0).minStorageBufferOffsetAlignment;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxVertexBuffers_78c71ff19beac74b() { return logError(function (arg0) {
    const ret = getObject(arg0).maxVertexBuffers;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxBufferSize_0c7ed57407582d40() { return logError(function (arg0) {
    const ret = getObject(arg0).maxBufferSize;
    return ret;
}, arguments) };

export function __wbg_maxVertexAttributes_c11cb018a9c5a224() { return logError(function (arg0) {
    const ret = getObject(arg0).maxVertexAttributes;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxVertexBufferArrayStride_c53560cc036cb477() { return logError(function (arg0) {
    const ret = getObject(arg0).maxVertexBufferArrayStride;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxInterStageShaderComponents_f9243ac86242eb18() { return logError(function (arg0) {
    const ret = getObject(arg0).maxInterStageShaderComponents;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxColorAttachments_d33b1d22c06a6fc5() { return logError(function (arg0) {
    const ret = getObject(arg0).maxColorAttachments;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxColorAttachmentBytesPerSample_637fd3ac394c14ee() { return logError(function (arg0) {
    const ret = getObject(arg0).maxColorAttachmentBytesPerSample;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxComputeWorkgroupStorageSize_7e5bc378e5a62367() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxComputeInvocationsPerWorkgroup_1ed5b24d52720f8a() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxComputeWorkgroupSizeX_56b713fb17f8c261() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeX;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxComputeWorkgroupSizeY_13040bdf12fd4e65() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeY;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxComputeWorkgroupSizeZ_8c8594730967472d() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_maxComputeWorkgroupsPerDimension_4094c8501eea36ce() { return logError(function (arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_createView_0ab0576f1665c9ad() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createView(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_destroy_57694ff5aabbf32d() { return logError(function (arg0) {
    getObject(arg0).destroy();
}, arguments) };

export function __wbg_instanceof_GpuValidationError_776dc042f9752ecb() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUValidationError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_getPreferredCanvasFormat_012ef9f3b0238ffa() { return logError(function (arg0) {
    const ret = getObject(arg0).getPreferredCanvasFormat();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_requestAdapter_e6f12701c7a38391() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAdapter(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_GpuAdapter_32bc80c8c30adaa0() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUAdapter;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_features_b56ebab8f515839e() { return logError(function (arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_limits_be2f592b5e154a3d() { return logError(function (arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_requestDevice_727ad8687b0d6553() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).requestDevice(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_GpuCanvasContext_b3bff0de75efe6fd() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUCanvasContext;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_configure_6cde48f0c99a3497() { return logError(function (arg0, arg1) {
    getObject(arg0).configure(getObject(arg1));
}, arguments) };

export function __wbg_getCurrentTexture_95b5b88416fdb0c2() { return logError(function (arg0) {
    const ret = getObject(arg0).getCurrentTexture();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_features_4991b2a28904a253() { return logError(function (arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_limits_1aa8a49e0a8442cc() { return logError(function (arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_queue_2bddd1700cb0bec2() { return logError(function (arg0) {
    const ret = getObject(arg0).queue;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_lost_42410660a8cd8819() { return logError(function (arg0) {
    const ret = getObject(arg0).lost;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_setonuncapturederror_4e4946a65c61f3ef() { return logError(function (arg0, arg1) {
    getObject(arg0).onuncapturederror = getObject(arg1);
}, arguments) };

export function __wbg_createBindGroup_2d6778f92445c8bf() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createBindGroup(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createBindGroupLayout_313b4151e718ff1f() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createBuffer_65c2fc555c46aa07() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createBuffer(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createCommandEncoder_1db1770ea9eab9af() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createComputePipeline_02674342979c6288() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createComputePipeline(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createPipelineLayout_9134c6c32c505ec8() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createQuerySet_424dbf8130140914() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createQuerySet(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createRenderBundleEncoder_32896e68340fabc6() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createRenderPipeline_2bfc852ce09914fc() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createSampler_942022241ecf4277() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createSampler(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createShaderModule_036b780a18124d9e() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createShaderModule(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_createTexture_5adbcf0db3fd41b4() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).createTexture(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_destroy_4f7ed2bbb4742899() { return logError(function (arg0) {
    getObject(arg0).destroy();
}, arguments) };

export function __wbg_popErrorScope_f8f0d4b6d5c635f9() { return logError(function (arg0) {
    const ret = getObject(arg0).popErrorScope();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_pushErrorScope_a09c8b037ab27e15() { return logError(function (arg0, arg1) {
    getObject(arg0).pushErrorScope(takeObject(arg1));
}, arguments) };

export function __wbg_end_e3cea1776c95d64f() { return logError(function (arg0) {
    getObject(arg0).end();
}, arguments) };

export function __wbg_executeBundles_16985086317c358a() { return logError(function (arg0, arg1) {
    getObject(arg0).executeBundles(getObject(arg1));
}, arguments) };

export function __wbg_setBlendConstant_496a0b5cc772c236() { return logError(function (arg0, arg1) {
    getObject(arg0).setBlendConstant(getObject(arg1));
}, arguments) };

export function __wbg_setScissorRect_9b7e673d03036c37() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
}, arguments) };

export function __wbg_setStencilReference_b4b1f7e586967a4d() { return logError(function (arg0, arg1) {
    getObject(arg0).setStencilReference(arg1 >>> 0);
}, arguments) };

export function __wbg_setViewport_85d18ceefd5180eb() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
}, arguments) };

export function __wbg_setBindGroup_c6ab2e9583489b58() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
}, arguments) };

export function __wbg_setBindGroup_0bf976b9657f99bd() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
}, arguments) };

export function __wbg_draw_540a514f996a5d0d() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
}, arguments) };

export function __wbg_drawIndexed_f717a07602ee2d18() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
}, arguments) };

export function __wbg_drawIndexedIndirect_bb5585ec7f45d269() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
}, arguments) };

export function __wbg_drawIndirect_c588ff54fb149aee() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
}, arguments) };

export function __wbg_setIndexBuffer_ea39707d8842fe03() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
}, arguments) };

export function __wbg_setIndexBuffer_04ba4ea48c8f80be() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
}, arguments) };

export function __wbg_setPipeline_d7c9c55035f118a6() { return logError(function (arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
}, arguments) };

export function __wbg_setVertexBuffer_907c60acf6dca161() { return logError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
}, arguments) };

export function __wbg_setVertexBuffer_9a336bb112a33317() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
}, arguments) };

export function __wbindgen_is_function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    _assertBoolean(ret);
    return ret;
};

export function __wbg_queueMicrotask_481971b0d87f3dd4() { return logError(function (arg0) {
    queueMicrotask(getObject(arg0));
}, arguments) };

export function __wbg_queueMicrotask_3cbae2ec6b6cd3d6() { return logError(function (arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_Window_f401953a2cf86220() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_document_5100775d18896c16() { return logError(function (arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_navigator_6c8fa55c5cc8796e() { return logError(function (arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_fetch_c4b6afebdb1f918e() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getElementById_c369ff43f0db99cf() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_querySelectorAll_4e0fcdb64cda2cd5() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).querySelectorAll(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_setAttribute_3c9f6c303b696daa() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_debug_5fb96680aecf5dc8() { return logError(function (arg0) {
    console.debug(getObject(arg0));
}, arguments) };

export function __wbg_error_8e3928cfb8a43e2b() { return logError(function (arg0) {
    console.error(getObject(arg0));
}, arguments) };

export function __wbg_info_530a29cb2e4e3304() { return logError(function (arg0) {
    console.info(getObject(arg0));
}, arguments) };

export function __wbg_log_5bb5f88f245d7762() { return logError(function (arg0) {
    console.log(getObject(arg0));
}, arguments) };

export function __wbg_warn_63bbae1730aead09() { return logError(function (arg0) {
    console.warn(getObject(arg0));
}, arguments) };

export function __wbg_instanceof_WorkerGlobalScope_46b577f151fad960() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof WorkerGlobalScope;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_navigator_56803b85352a0575() { return logError(function (arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_fetch_921fad6ef9e883dd() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_Response_849eb93e75734b6e() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_status_61a01141acd3cf74() { return logError(function (arg0) {
    const ret = getObject(arg0).status;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_text_450a059667fd91fd() { return handleError(function (arg0) {
    const ret = getObject(arg0).text();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_ab6fd82b10560829() { return handleError(function () {
    const ret = new Headers();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_HtmlCanvasElement_46bdbf323b0b18d1() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLCanvasElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_setwidth_080107476e633963() { return logError(function (arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
}, arguments) };

export function __wbg_setheight_dc240617639f1f51() { return logError(function (arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
}, arguments) };

export function __wbg_getContext_df50fa48a8876636() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_search_c68f506c44be6d1e() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).search;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_setsearch_fd62f4de409a2bb3() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).search = getStringFromWasm0(arg1, arg2);
}, arguments) };

export function __wbg_new_67853c351755d2cf() { return handleError(function (arg0, arg1) {
    const ret = new URL(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_4c501d7c115d20a6() { return handleError(function () {
    const ret = new URLSearchParams();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_get_8cd5eba00ab6304f() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_setwidth_83d936c4b04dcbec() { return logError(function (arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
}, arguments) };

export function __wbg_setheight_6025ba0d58e6cc8c() { return logError(function (arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
}, arguments) };

export function __wbg_getContext_c102f659d540d068() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_now_4e659b3d15f470d9() { return logError(function (arg0) {
    const ret = getObject(arg0).now();
    return ret;
}, arguments) };

export function __wbg_url_7807f6a1fddc3e23() { return logError(function (arg0, arg1) {
    const ret = getObject(arg1).url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_newwithstr_36b0b3f97efe096f() { return handleError(function (arg0, arg1) {
    const ret = new Request(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_newwithstrandinit_3fd6fba4083ff2d0() { return handleError(function (arg0, arg1, arg2) {
    const ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_16b304a2cfa7ff4a() { return logError(function () {
    const ret = new Array();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_push_a5b05aedc7234f9f() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_instanceof_Error_e20bb56fd5591a93() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Error;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_message_5bf28016c2b49cfb() { return logError(function (arg0) {
    const ret = getObject(arg0).message;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_name_e7429f0dda6079e2() { return logError(function (arg0) {
    const ret = getObject(arg0).name;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_toString_ffe4c9ea3b3532e9() { return logError(function (arg0) {
    const ret = getObject(arg0).toString();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_newnoargs_e258087cd0daa0ea() { return logError(function (arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_call_27c0f87801dedf93() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_call_b3ca7c6051f9bec1() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_Object_71ca3c0a59266746() { return logError(function (arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Object;
    } catch (_) {
        result = false;
    }
    const ret = result;
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbg_new_72fb9a18b5ae2624() { return logError(function () {
    const ret = new Object();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_toString_c816a20ab859d0c1() { return logError(function (arg0) {
    const ret = getObject(arg0).toString();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_valueOf_a0b7c836f68a054b() { return logError(function (arg0) {
    const ret = getObject(arg0).valueOf();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_81740750da40724f() { return logError(function (arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_423(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = state0.b = 0;
    }
}, arguments) };

export function __wbg_resolve_b0083a7967828ec8() { return logError(function (arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_then_0c86a60e8fcfe9f6() { return logError(function (arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_then_a73caa9a87991566() { return logError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_globalThis_d1e6af4856ba331b() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_self_ce0dbfc45cf2f5be() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_window_c6fb939a7f436783() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_global_207b558942527489() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_63b92bc8671ed464() { return logError(function (arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb() { return logError(function (arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_buffer_dd7f74bc60f1faab() { return logError(function (arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_length_c20a40f15020d68a() { return logError(function (arg0) {
    const ret = getObject(arg0).length;
    _assertNum(ret);
    return ret;
}, arguments) };

export function __wbg_set_a47bac70306a19a7() { return logError(function (arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
}, arguments) };

export function __wbg_buffer_12d079cc21e14bdb() { return logError(function (arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_get_e3c254076557e348() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_set_1f9b04f170055d33() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    _assertBoolean(ret);
    return ret;
}, arguments) };

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbindgen_memory() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper16723() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1257, __wbg_adapter_26);
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_closure_wrapper16989() { return logError(function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1261, __wbg_adapter_29);
    return addHeapObject(ret);
}, arguments) };

