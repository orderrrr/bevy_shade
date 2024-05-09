<script>
  import init from "/shade/bevy_shade.js?url";
  import { push_shader } from '/js_reader.js?url';
  import { onMount } from "svelte";

  // so does work.
  // push_shader("shaders/fragment.wgsl", "BROKEN");

  onMount(() => {
    init().catch((error) => {
      if (
        !error.message.startsWith(
          "Using exceptions for control flow, don't mind me. This isn't actually an error!"
        )
      ) {
        throw error;
      }
    });
  });

  const click = () => {
    push_shader("shaders/fragment.wgsl", "BROKEN");
  }
</script>

<canvas id="bevy_shade_canvas" width="1280" height="720"></canvas>

<button on:click={click}>Click me</button>
