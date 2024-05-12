<script>
	import Header from "$lib/header/Header.svelte";
	import "../app.css";
</script>

<canvas id="bevy_shade_canvas"></canvas>

<Header />

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
