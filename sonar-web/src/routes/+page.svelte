<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;
</script>

{#each data.artists as artist (artist.id)}
	{artist.name}
{/each}
