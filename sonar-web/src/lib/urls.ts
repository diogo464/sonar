export function imageIdToUrl(imageId: string): string {
	return `/image/${imageId}`;
}

export function artistIdToUrl(artistId: string): string {
	return `/artist/${artistId}`;
}

export function albumIdToUrl(albumId: string): string {
	return `/album/${albumId}`;
}

export function trackIdToUrl(trackId: string): string {
	return `/track/${trackId}`;
}
