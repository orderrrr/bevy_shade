<script>
    import { onMount } from "svelte";
    import { createEventDispatcher } from "svelte";
    import { createWgpuToyRenderer } from "$lib/wgputoy";

    import { watchResize } from "svelte-watch-resize";

    import {
        canvas_el,
        wgpu_availability,
        wgpu_renderer,
        resize,
    } from "./stores.js";

    const dispatch = createEventDispatcher();

    export let bindID = "wgputoy-canvas";
    export let style = {};

    let canvas;
    let loaded = false;

    onMount(async () => {
        if (canvas && canvas.getContext("webgpu") && "gpu" in navigator) {
            try {
                const adapter = await navigator.gpu.requestAdapter();
                if (adapter) {
                    const device = await adapter.requestDevice();
                    if (device) {
                        wgpu_availability.set("available");
                        canvas_el.set(canvas);
                        loaded = true;
                        console.log("Setting wgpu renderer");
                        wgpu_renderer.set(
                            await createWgpuToyRenderer(canvas.id),
                        );
                        dispatch("ready", "WebGPU is available");
                    } else {
                        wgpu_availability.set("unavailable");
                    }
                } else {
                    wgpu_availability.set("unavailable");
                }
            } catch (error) {
                console.error("Error setting up WebGPU:", error);
                wgpu_availability.set("unavailable");
            }
        } else {
            wgpu_availability.set("unavailable");
        }
    });

    let resizeTimer;

    const debounce = (func, delay) => {
        return function () {
            clearTimeout(resizeTimer);
            resizeTimer = setTimeout(func, delay);
        };
    };

    const watch_resize = debounce((_, _callback) => {
        resize.set(true);
        console.log("resize");

        // Reset the resize flag after a short delay
        setTimeout(() => resize.set(false), 100);
    }, 250);

    // Set up the event listener
    if (typeof window !== "undefined") {
        window.addEventListener("resize", watch_resize);
    }

    $: canvasStyle = loaded
        ? {
              ...style,
              outline: "none",
              position: "fixed",
              width: "100%",
              height: "100%",
          }
        : {
              position: "fixed",
              display: "none",
              width: "100%",
              height: "100%",
          };
</script>

<div use:watchResize={watch_resize}>
    <canvas
        bind:this={canvas}
        id={bindID}
        style={Object.entries(canvasStyle)
            .map(([key, value]) => `${key}: ${value};`)
            .join(" ")}
        tabindex="0"
    ></canvas>
</div>

{#if $wgpu_availability === "unknown"}
    <div>Loading...</div>
{:else if !loaded}
    <div>
        <p>WebGPU support was not detected in your browser.</p>
        <p>
            For information on how to set up your browser to run WebGPU code,
            please see the instructions linked on the homepage.
        </p>
    </div>
{/if}

<style>
    #canvas {
        position: fixed;
        display: none;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: all;
    }
</style>
