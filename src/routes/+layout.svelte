<script>
    import Canvas from "$lib/canvas/Canvas.svelte";
    import "../app.css";

    const ready = (e) => {
        console.log(e);
    };
</script>

<Canvas on:message={ready} />

<main>
    <slot />
</main>

<style>
    main {
        flex: 1;
        display: flex;
        flex-direction: column;
        width: 100%;
        box-sizing: border-box;
    }
</style>
