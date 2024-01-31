<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { artistIdToUrl } from '$lib/urls';

	export let data: PageData;
	export let form: ActionData;
</script>

<table class="w-full text-left text-white">
	<thead>
		<tr>
			<th>ID</th>
			<th>Name</th>
			<th>Listen Count</th>
			<th>Album Count</th>
		</tr>
	</thead>
	<tbody>
		{#each data.artists as artist (artist.id)}
			<tr>
				<td><a href={artistIdToUrl(artist.id)}>{artist.id}</a></td>
				<td>{artist.name}</td>
				<td>{artist.listenCount}</td>
				<td>{artist.albumCount}</td>
			</tr>
		{/each}
	</tbody>
</table>
