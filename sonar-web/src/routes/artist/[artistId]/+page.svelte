<script lang="ts">
	import { imageIdToUrl, albumIdToUrl } from '$lib/urls';
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	const artist = data.artist;
	const albums = data.albums;
</script>

<h1 class="text-4xl font-bold text-white">{artist.name}</h1>
{#if artist.coverartId}
	<img class="h-64 w-64" src={imageIdToUrl(artist.coverartId)} alt={artist.name} />
{/if}

<h2 class="text-2xl font-bold text-white pt-4">Albums</h2>

<table class="w-full text-left text-white">
	<thead>
		<tr>
			<th>ID</th>
			<th>Name</th>
			<th>Listen Count</th>
			<th>Track Count</th>
		</tr>
	</thead>
	<tbody>
		{#each albums as album (album.id)}
			<tr>
				<td><a href={albumIdToUrl(album.id)}>{album.id}</a></td>
				<td>{album.name}</td>
				<td>{album.listenCount}</td>
				<td>{album.trackCount}</td>
			</tr>
		{/each}
	</tbody>
</table>
