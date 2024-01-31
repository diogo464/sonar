import type { Action } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import { createSonarClientFromCookies } from '$lib/server';

export const load: PageServerLoad = async ({ cookies, request }) => {
	const client = createSonarClientFromCookies(cookies);
	const response = await client.artistList({})
	return { artists: response.artists };
};
