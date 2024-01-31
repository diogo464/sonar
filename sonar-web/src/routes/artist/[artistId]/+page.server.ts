import type { Action } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import { createSonarClientFromCookies } from '$lib/server';

export const load: PageServerLoad = async ({ params, cookies, request }) => {
	const client = createSonarClientFromCookies(cookies);
	const artist = await client.artistGet({ artistId: params.artistId });
	const albums = (await client.albumListByArtist({ artistId: params.artistId })).albums;
	return { artist, albums };
};
