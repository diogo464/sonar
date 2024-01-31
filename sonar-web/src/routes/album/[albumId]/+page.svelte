<script lang="ts">
	import { imageIdToUrl, trackIdToUrl } from '$lib/urls';
	import { displayDuration, propertiesGetTrackNumber } from '$lib/util';
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	const album = data.album;
	let tracks = data.tracks;
	tracks.sort(
		(a, b) =>
			(propertiesGetTrackNumber(a.properties) || 0) - (propertiesGetTrackNumber(b.properties) || 0)
	);
</script>

<h1 class="text-4xl font-bold text-white">{album.name}</h1>
{#if album.coverartId}
	<img class="h-64 w-64" src={imageIdToUrl(album.coverartId)} alt={album.name} />
{/if}

<h2 class="pt-4 text-2xl font-bold text-white">Tracks</h2>

<table class="w-full text-left text-white">
	<thead>
		<tr>
			<th>ID</th>
			<th>Number</th>
			<th>Name</th>
			<th>Listen Count</th>
			<th>Duration</th>
		</tr>
	</thead>
	<tbody>
		{#each tracks as track (track.id)}
			<tr>
				<td><a href={trackIdToUrl(track.id)}>{track.id}</a></td>
				<td>{propertiesGetTrackNumber(track.properties)}</td>
				<td>{track.name}</td>
				<td>{track.listenCount}</td>
				<td>{displayDuration(track.duration)}</td>
			</tr>
		{/each}
	</tbody>
</table>
