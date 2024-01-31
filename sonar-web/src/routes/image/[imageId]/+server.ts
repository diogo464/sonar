import { createSonarClientFromCookies } from '$lib/server';
import type { RequestHandler } from './$types';
import { error, fail } from '@sveltejs/kit';

export const GET: RequestHandler = async ({ request, params, url, cookies }) => {
	const imageId = params.imageId;
	const client = createSonarClientFromCookies(cookies);
	const download = client.imageDownload({ imageId });

	let mimeType = "";
	let arrays = [];
	for await (const chunk of download) {
		mimeType = chunk.mimeType;
		arrays.push(chunk.content);
	}

	const content = new Uint8Array(arrays.reduce((a, b) => a + b.length, 0));
	let offset = 0;
	for (const array of arrays) {
		content.set(array, offset);
		offset += array.length;
	}

	return new Response(content, {
		headers: {
			'content-type': mimeType,
			'cache-control': 'public, max-age=86400',
		},
	});
}
