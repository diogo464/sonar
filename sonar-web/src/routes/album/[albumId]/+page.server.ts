import type { Action } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import { createSonarClientFromCookies } from '$lib/server';

export const load: PageServerLoad = async ({ params, cookies, request }) => {
	const client = createSonarClientFromCookies(cookies);
	const album = await client.albumGet({ albumId: params.albumId });
	const tracks = (await client.trackListByAlbum({ albumId: params.albumId })).tracks;
	return { album, tracks };
};

