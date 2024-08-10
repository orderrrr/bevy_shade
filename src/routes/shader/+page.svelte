<script>
    import { onMount } from "svelte";
    import { writable, derived } from "svelte/store";
    import CodeMirror from "svelte-codemirror-editor";
    import { basicSetup } from "codemirror";
    import { oneDark } from "@codemirror/theme-one-dark";
    import { wgsl } from "@iizukak/codemirror-lang-wgsl";
    import { vim } from "@replit/codemirror-vim";
    import {
        wgpuRenderer,
        needsInitialReset,
        parseError,
        manualReload,
        isSafeContext,
        playing,
    } from "$lib/canvas/stores.js";

    let value = `
        @compute @workgroup_size(16, 16)
        fn main_image(@builtin(global_invocation_id) id: vec3u) {
            // Viewport resolution (in pixels)
            let screen_size = textureDimensions(screen);

            // Prevent overdraw for workgroups on the edge of the viewport
            if (id.x >= screen_size.x || id.y >= screen_size.y) { return; }

            // Pixel coordinates (centre of pixel, origin at bottom left)
            let fragCoord = vec2f(f32(id.x) + .5, f32(screen_size.y - id.y) - .5);

            // Normalised pixel coordinates (from 0 to 1)
            let uv = fragCoord / vec2f(screen_size);

            // Time varying pixel colour
            var col = .5 + .2 * cos(time.elapsed + uv.xyx + vec3f(0.,2.,4.));

            // Convert from gamma-encoded to linear colour space
            col = pow(col, vec3f(2.2));

            // Output to screen (linear colour space)
            textureStore(screen, id.xy, vec4f(col, 1.));
        }`;

    let wgputoy = null;

    const handle_change = async () => {
        if (isSafeContext(wgputoy)) {
            reload();
        }
        // push_shader("shaders/fragment.wgsl", value);
    };

    const reload = () => {
        updateUniforms().then(() => {
            if (isSafeContext(wgputoy)) {
                wgputoy.preprocess(value).then((s) => {
                    if (s) {
                        wgputoy.compile(s);
                        // setPrelude(wgputoy.prelude()); // TODO: this is not working
                        wgputoy.render();
                    }
                });
                manualReload.set(false);
            }
        });
    };

    const updateUniforms = async () => {
        if (isSafeContext(wgputoy)) {
            const names = [];
            const values = [];
            // [...sliderRefMap().keys()].map((uuid) => {
            //     names.push(sliderRefMap().get(uuid).getUniform());
            //     values.push(sliderRefMap().get(uuid).getVal());
            // }, this);
            if (names.length > 0) {
                await wgputoy.set_custom_floats(
                    names,
                    Float32Array.from(values),
                );
            }
            // setSliderUpdateSignal(false);
        }
    };

    const awaitableReload = async () => {
        await updateUniforms();
        if (isSafeContext(wgputoy)) {
            const s = await wgputoy.preprocess(value);
            if (s) {
                wgputoy.compile(s);
                //     // setPrelude(wgputoy.prelude()); // TODO: this is not working
                wgputoy.render();
            }
            return true;
        } else {
            return false;
        }
    };

    const reset = () => {
        if (isSafeContext(wgputoy)) {
            wgputoy.reset();
            reload();
        }
    };

    const liveReload = async () => {
        if ($needsInitialReset) {
            console.log("needs initial reset");
            const ready = await awaitableReload();
            if (ready && $parseError.success) {
                reset();
                needsInitialReset.set(false);
            }
        } else if ($manualReload) {
            console.log("manual reload");
            reload();
        }
    };

    const handleSuccess = (entryPoints) => {
        // setEntryPoints(entryPoints);
        parseError.set({
            success: true,
        });
    };

    const handleError = (summary, row, col) => {
        parseError.set(() => ({
            summary: summary,
            position: { row: Number(row), col: Number(col) },
            success: false,
        }));

        console.log("error", summary, row, col);
    };

    $: if ($wgpuRenderer) {
        wgputoy = $wgpuRenderer;

        let lastTimestamp = 0;
        let timer = 0;

        const loop = async (e) => {
            requestAnimationFrame(loop);

            await liveReload();

            const deltaTime = (e - lastTimestamp) / 1000;
            lastTimestamp = e;
            timer += deltaTime;

            wgputoy.set_time_delta(deltaTime);
            wgputoy.set_time_elapsed(timer);

            wgputoy.render();
        };

        wgputoy.on_success(handleSuccess);
        if (window) window["wgsl_error_handler"] = handleError;

        requestAnimationFrame(loop);
    }
</script>

<CodeMirror
    class="text_edit"
    bind:value
    extensions={[vim(), basicSetup]}
    lang={wgsl()}
    theme={oneDark}
    on:change={handle_change}
/>

<style>
    .text_edit {
        position: absolute;
    }
</style>
