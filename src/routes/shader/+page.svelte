<script>
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
    } from "$lib/canvas/stores.js";
    import { compress, decompress } from "./util.js";

    let default_shader = `G0sDABwFbtNzOZJEv8ZQJDrMt2bR/j7ViXsPYArkFxPExQxycT6i1UW7Unjv4HL6sxJ84pS61Zo0t45F99anGwNcgHnIaebWHkEGPdOx3eZwEapRB5POBGUkpUSXLhh7pFpjVnqHMHaZCiUwGtNOjOYK9sfR1J5X0DLfPY+u0e/MVwDI8thx1SJFCHLDypWRDtbuBoQEoIYQ2Jv7pfs2ckyDsZyZSw8V1AzXlklo0rvi2L8PU3wi638AmHoQrfRCFG6QWf39FEZ9D9/M94Hm3h8LKqPv6IM0YoEG2/LfIZYQhvZX+s/6SzXkTMoIbTin9e51DeAOw0M6QZ4fTwUizioj0mjL8FRPb0zXrsUJHEaHB4o/oCK4vz8QwnBkn0gtA+OYNOCSZG0ZJMJCzJohJB36atBn+Uq+NpO6tTph+8Dj/pBe7xd4h/m1tyIJkYSVmCzp9kSol4Zq2yrxHYEkfLL0DOa9bgJS93Q86SoniIgdAW3nubuQE2t5DMSRLSSeSL30xCZlQdWU3wJ8w9hpjlWgxiIM`;
    var params = window.location.hash.substring(1);

    if (params.length > 0 || params == null || params == "") {
        params = default_shader;
    }

    let param_promise = decompress(params);
    let value = "";

    param_promise.then((promise) => {
        value = promise;
        handle_change();
    });

    let wgputoy = null;

    const handle_change = async () => {
        if (isSafeContext(wgputoy)) {
            reload();
        }

        window.location.hash = await compress(value);
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

    $: if ($wgpuRenderer && param_promise) {
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

{#if param_promise}
    <CodeMirror
        class="text_edit"
        bind:value
        extensions={[vim(), basicSetup]}
        lang={wgsl()}
        theme={oneDark}
        on:change={handle_change}
    />
{/if}

<style>
    .text_edit {
        position: absolute;
    }
</style>
