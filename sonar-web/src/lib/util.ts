import type { Property } from "./server";
import type { Duration } from "./server/pb/google/protobuf/duration";

export const PROPERTY_TRACK_NUMBER = "sonar.io/track-number";
export const PROPERTY_DISC_NUMBER = "sonar.io/disc-number";

export function displayDuration(dur: Duration | undefined): string {
	if (!dur) return '-';
	const totalSeconds = dur.seconds + dur.nanos / 1e9;
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = Math.floor(totalSeconds % 60);
	return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

export function propertiesGetTrackNumber(properties: Property[]): number | undefined {
	const property = properties.find(p => p.key === PROPERTY_TRACK_NUMBER);
	if (!property) return undefined;
	return Number(property.value);
}

export function propertiesGetDiscNumber(properties: Property[]): number | undefined {
	const property = properties.find(p => p.key === PROPERTY_DISC_NUMBER);
	if (!property) return undefined;
	return Number(property.value);
}
