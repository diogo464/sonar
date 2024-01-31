import type { SonarServiceClient } from './pb/sonar';
import { SonarServiceDefinition } from './pb/sonar';
import { redirect, type Cookies } from '@sveltejs/kit';
import { Metadata, createChannel, createClientFactory } from 'nice-grpc';
// import { env } from '$env/dynamic/private';

export type * from './pb/sonar';
export * from './pb/sonar';

export function createSonarClient(token: string): SonarServiceClient {
	const address = "http://127.0.0.1:3000";
	console.log(`connecting to ${address}`);
	const channel = createChannel(address);
	const client = createClientFactory().use((call, options) => {
		options = { ...options, metadata: Metadata(options.metadata).set('authorization', token) };
		return call.next(call.request, options)
	}).create(SonarServiceDefinition, channel);
	return client;
}

export function createSonarClientFromCookies(cookies: Cookies): SonarServiceClient {
	const token = cookies.get('authorization');
	if (!token)
		throw redirect(303, '/login');
	return createSonarClient(token);
}
