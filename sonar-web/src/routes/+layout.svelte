<script>
	import NavBar from '$lib/component/NavBar.svelte';
	import '../app.pcss';
</script>

<div class="min-h-screen w-screen bg-gray-800">
	<NavBar />
	<slot />
</div>
