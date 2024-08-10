<script>
    import { onMount } from "svelte";
    import { canvasEl, wgpuAvailability, wgpuRenderer } from "./stores.js";
    import { createEventDispatcher } from "svelte";
    import { createWgpuToyRenderer } from "$lib/wgputoy";

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
                        wgpuAvailability.set("available");
                        canvasEl.set(canvas);
                        loaded = true;
                        wgpuRenderer.set(
                            await createWgpuToyRenderer(
                                canvas.width,
                                canvas.height,
                                canvas.id,
                            ),
                        );
                        dispatch("ready", "WebGPU is available");
                    } else {
                        wgpuAvailability.set("unavailable");
                    }
                } else {
                    wgpuAvailability.set("unavailable");
                }
            } catch (error) {
                console.error("Error setting up WebGPU:", error);
                wgpuAvailability.set("unavailable");
            }
        } else {
            wgpuAvailability.set("unavailable");
        }
    });

    $: canvasStyle = loaded
        ? {
              ...style,
              outline: "none",
              position: "absolute",
              width: "100%",
              height: "100%",
          }
        : {
              position: "absolute",
              display: "none",
              width: "100%",
              height: "100%",
          };
</script>

<canvas
    bind:this={canvas}
    id={bindID}
    style={Object.entries(canvasStyle)
        .map(([key, value]) => `${key}: ${value};`)
        .join(" ")}
    tabindex="0"
/>

{#if $wgpuAvailability === "unknown"}
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
        position: absolute;
        display: none;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
    }
</style>
