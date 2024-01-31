/* eslint-disable */
import type { CallContext, CallOptions } from "nice-grpc-common";
import _m0 from "protobufjs/minimal";
import { Duration } from "./google/protobuf/duration";
import { Empty } from "./google/protobuf/empty";
import { FieldMask } from "./google/protobuf/field_mask";
import { Timestamp } from "./google/protobuf/timestamp";

export const protobufPackage = "sonar";

export enum MetadataFetchKind {
  METADATA_FETCH_KIND_ARTIST = 0,
  METADATA_FETCH_KIND_ALBUM = 1,
  METADATA_FETCH_KIND_ALBUMTRACKS = 2,
  METADATA_FETCH_KIND_TRACK = 3,
  UNRECOGNIZED = -1,
}

export function metadataFetchKindFromJSON(object: any): MetadataFetchKind {
  switch (object) {
    case 0:
    case "METADATA_FETCH_KIND_ARTIST":
      return MetadataFetchKind.METADATA_FETCH_KIND_ARTIST;
    case 1:
    case "METADATA_FETCH_KIND_ALBUM":
      return MetadataFetchKind.METADATA_FETCH_KIND_ALBUM;
    case 2:
    case "METADATA_FETCH_KIND_ALBUMTRACKS":
      return MetadataFetchKind.METADATA_FETCH_KIND_ALBUMTRACKS;
    case 3:
    case "METADATA_FETCH_KIND_TRACK":
      return MetadataFetchKind.METADATA_FETCH_KIND_TRACK;
    case -1:
    case "UNRECOGNIZED":
    default:
      return MetadataFetchKind.UNRECOGNIZED;
  }
}

export function metadataFetchKindToJSON(object: MetadataFetchKind): string {
  switch (object) {
    case MetadataFetchKind.METADATA_FETCH_KIND_ARTIST:
      return "METADATA_FETCH_KIND_ARTIST";
    case MetadataFetchKind.METADATA_FETCH_KIND_ALBUM:
      return "METADATA_FETCH_KIND_ALBUM";
    case MetadataFetchKind.METADATA_FETCH_KIND_ALBUMTRACKS:
      return "METADATA_FETCH_KIND_ALBUMTRACKS";
    case MetadataFetchKind.METADATA_FETCH_KIND_TRACK:
      return "METADATA_FETCH_KIND_TRACK";
    case MetadataFetchKind.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

export interface Property {
  key: string;
  value: string;
}

export interface PropertyUpdate {
  key: string;
  value?: string | undefined;
}

export interface User {
  userId: string;
  username: string;
  avatarId?: string | undefined;
}

export interface UserListRequest {
  offset?: number | undefined;
  count?: number | undefined;
}

export interface UserListResponse {
  users: User[];
}

export interface UserCreateRequest {
  username: string;
  password: string;
  avatarId?: string | undefined;
}

export interface UserUpdateRequest {
  userId: string;
  password?: string | undefined;
  avatarId?: string | undefined;
}

export interface UserDeleteRequest {
  userId: string;
}

export interface ImageCreateRequest {
  content: Uint8Array;
}

export interface ImageCreateResponse {
  imageId: string;
}

export interface ImageDeleteRequest {
  imageId: string;
}

export interface ImageDownloadRequest {
  imageId: string;
}

export interface ImageDownloadResponse {
  imageId: string;
  mimeType: string;
  content: Uint8Array;
}

export interface Artist {
  id: string;
  name: string;
  albumCount: number;
  listenCount: number;
  coverartId?: string | undefined;
  properties: Property[];
}

export interface ArtistListRequest {
  offset?: number | undefined;
  count?: number | undefined;
}

export interface ArtistListResponse {
  artists: Artist[];
}

export interface ArtistGetRequest {
  artistId: string;
}

export interface ArtistCreateRequest {
  name: string;
  coverartId?: string | undefined;
  properties: Property[];
}

export interface ArtistDeleteRequest {
  artistId: string;
}

export interface ArtistUpdateRequest {
  artistId: string;
  name?: string | undefined;
  properties: PropertyUpdate[];
  mask: string[] | undefined;
}

export interface Album {
  id: string;
  name: string;
  trackCount: number;
  duration: Duration | undefined;
  listenCount: number;
  artists: number[];
  coverartId?: string | undefined;
  properties: Property[];
}

export interface AlbumListRequest {
  offset?: number | undefined;
  count?: number | undefined;
}

export interface AlbumListByArtistRequest {
  offset?: number | undefined;
  count?: number | undefined;
  artistId: string;
}

export interface AlbumListResponse {
  albums: Album[];
}

export interface AlbumGetRequest {
  albumId: string;
}

export interface AlbumCreateRequest {
  name: string;
  artistId: string;
  coverartId?: string | undefined;
  properties: Property[];
}

export interface AlbumUpdateRequest {
  albumId: string;
  name?: string | undefined;
  artistId?: string | undefined;
  coverartId?: string | undefined;
  properties: PropertyUpdate[];
}

export interface AlbumDeleteRequest {
  albumId: string;
}

export interface Track {
  id: string;
  name: string;
  artistId: number;
  albumId: number;
  duration: Duration | undefined;
  listenCount: number;
  coverArtId?: string | undefined;
  properties: Property[];
}

export interface TrackListRequest {
  offset?: number | undefined;
  count?: number | undefined;
}

export interface TrackListByAlbumRequest {
  offset?: number | undefined;
  count?: number | undefined;
  albumId: string;
}

export interface TrackListResponse {
  tracks: Track[];
}

export interface TrackGetRequest {
  trackId: string;
}

export interface TrackCreateRequest {
  name: string;
  albumId: string;
  coverartId?: string | undefined;
  audioId?: string | undefined;
  properties: Property[];
}

export interface TrackUpdateRequest {
  trackId: string;
  name?: string | undefined;
  albumId?: string | undefined;
  coverartId?: string | undefined;
  properties: PropertyUpdate[];
}

export interface TrackDeleteRequest {
  trackId: string;
}

export interface Playlist {
  id: string;
  name: string;
  userId: string;
  trackCount: number;
  duration: Duration | undefined;
  coverartId?: string | undefined;
  properties: Property[];
}

export interface PlaylistListRequest {
  offset?: number | undefined;
  count?: number | undefined;
}

export interface PlaylistListResponse {
  playlists: Playlist[];
}

export interface PlaylistGetRequest {
  playlistId: string;
}

export interface PlaylistCreateRequest {
  name: string;
  ownerId: string;
  trackIds: string[];
  properties: Property[];
}

export interface PlaylistUpdateRequest {
  playlistId: string;
  name?: string | undefined;
  properties: PropertyUpdate[];
}

export interface PlaylistDeleteRequest {
  playlistId: string;
}

export interface PlaylistTrackListRequest {
  playlistId: string;
}

export interface PlaylistTrackListResponse {
  tracks: Track[];
}

export interface PlaylistTrackInsertRequest {
  playlistId: string;
  trackIds: string[];
}

export interface PlaylistTrackRemoveRequest {
  playlistId: string;
  trackIds: string[];
}

export interface PlaylistTrackClearRequest {
  playlistId: string;
}

export interface Scrobble {
  id: string;
  trackId: string;
  userId: string;
  duration: Duration | undefined;
  timestamp: Date | undefined;
  properties: Property[];
}

export interface ImportRequest {
  chunk: Uint8Array;
  filepath?: string | undefined;
  artistId?: number | undefined;
  albumId?: number | undefined;
}

export interface MetadataFetchRequest {
  kind: MetadataFetchKind;
  itemId: number;
}

export interface TrackMetadata {
  name?: string | undefined;
  properties: Property[];
  cover?: Uint8Array | undefined;
}

export interface MetadataAlbumTracksRequest {
  albumId: number;
}

export interface MetadataAlbumTracksResponse {
  tracks: { [key: number]: TrackMetadata };
}

export interface MetadataAlbumTracksResponse_TracksEntry {
  key: number;
  value: TrackMetadata | undefined;
}

function createBaseProperty(): Property {
  return { key: "", value: "" };
}

export const Property = {
  encode(message: Property, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key);
    }
    if (message.value !== "") {
      writer.uint32(18).string(message.value);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Property {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseProperty();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.key = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.value = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Property {
    return {
      key: isSet(object.key) ? globalThis.String(object.key) : "",
      value: isSet(object.value) ? globalThis.String(object.value) : "",
    };
  },

  toJSON(message: Property): unknown {
    const obj: any = {};
    if (message.key !== "") {
      obj.key = message.key;
    }
    if (message.value !== "") {
      obj.value = message.value;
    }
    return obj;
  },

  create(base?: DeepPartial<Property>): Property {
    return Property.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<Property>): Property {
    const message = createBaseProperty();
    message.key = object.key ?? "";
    message.value = object.value ?? "";
    return message;
  },
};

function createBasePropertyUpdate(): PropertyUpdate {
  return { key: "", value: undefined };
}

export const PropertyUpdate = {
  encode(message: PropertyUpdate, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key);
    }
    if (message.value !== undefined) {
      writer.uint32(18).string(message.value);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PropertyUpdate {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePropertyUpdate();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.key = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.value = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PropertyUpdate {
    return {
      key: isSet(object.key) ? globalThis.String(object.key) : "",
      value: isSet(object.value) ? globalThis.String(object.value) : undefined,
    };
  },

  toJSON(message: PropertyUpdate): unknown {
    const obj: any = {};
    if (message.key !== "") {
      obj.key = message.key;
    }
    if (message.value !== undefined) {
      obj.value = message.value;
    }
    return obj;
  },

  create(base?: DeepPartial<PropertyUpdate>): PropertyUpdate {
    return PropertyUpdate.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PropertyUpdate>): PropertyUpdate {
    const message = createBasePropertyUpdate();
    message.key = object.key ?? "";
    message.value = object.value ?? undefined;
    return message;
  },
};

function createBaseUser(): User {
  return { userId: "", username: "", avatarId: undefined };
}

export const User = {
  encode(message: User, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userId !== "") {
      writer.uint32(10).string(message.userId);
    }
    if (message.username !== "") {
      writer.uint32(18).string(message.username);
    }
    if (message.avatarId !== undefined) {
      writer.uint32(26).string(message.avatarId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): User {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUser();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.userId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.username = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.avatarId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): User {
    return {
      userId: isSet(object.userId) ? globalThis.String(object.userId) : "",
      username: isSet(object.username) ? globalThis.String(object.username) : "",
      avatarId: isSet(object.avatarId) ? globalThis.String(object.avatarId) : undefined,
    };
  },

  toJSON(message: User): unknown {
    const obj: any = {};
    if (message.userId !== "") {
      obj.userId = message.userId;
    }
    if (message.username !== "") {
      obj.username = message.username;
    }
    if (message.avatarId !== undefined) {
      obj.avatarId = message.avatarId;
    }
    return obj;
  },

  create(base?: DeepPartial<User>): User {
    return User.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<User>): User {
    const message = createBaseUser();
    message.userId = object.userId ?? "";
    message.username = object.username ?? "";
    message.avatarId = object.avatarId ?? undefined;
    return message;
  },
};

function createBaseUserListRequest(): UserListRequest {
  return { offset: undefined, count: undefined };
}

export const UserListRequest = {
  encode(message: UserListRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserListRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUserListRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): UserListRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
    };
  },

  toJSON(message: UserListRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    return obj;
  },

  create(base?: DeepPartial<UserListRequest>): UserListRequest {
    return UserListRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<UserListRequest>): UserListRequest {
    const message = createBaseUserListRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    return message;
  },
};

function createBaseUserListResponse(): UserListResponse {
  return { users: [] };
}

export const UserListResponse = {
  encode(message: UserListResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.users) {
      User.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserListResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUserListResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.users.push(User.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): UserListResponse {
    return { users: globalThis.Array.isArray(object?.users) ? object.users.map((e: any) => User.fromJSON(e)) : [] };
  },

  toJSON(message: UserListResponse): unknown {
    const obj: any = {};
    if (message.users?.length) {
      obj.users = message.users.map((e) => User.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<UserListResponse>): UserListResponse {
    return UserListResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<UserListResponse>): UserListResponse {
    const message = createBaseUserListResponse();
    message.users = object.users?.map((e) => User.fromPartial(e)) || [];
    return message;
  },
};

function createBaseUserCreateRequest(): UserCreateRequest {
  return { username: "", password: "", avatarId: undefined };
}

export const UserCreateRequest = {
  encode(message: UserCreateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.username !== "") {
      writer.uint32(10).string(message.username);
    }
    if (message.password !== "") {
      writer.uint32(18).string(message.password);
    }
    if (message.avatarId !== undefined) {
      writer.uint32(26).string(message.avatarId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserCreateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUserCreateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.username = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.password = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.avatarId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): UserCreateRequest {
    return {
      username: isSet(object.username) ? globalThis.String(object.username) : "",
      password: isSet(object.password) ? globalThis.String(object.password) : "",
      avatarId: isSet(object.avatarId) ? globalThis.String(object.avatarId) : undefined,
    };
  },

  toJSON(message: UserCreateRequest): unknown {
    const obj: any = {};
    if (message.username !== "") {
      obj.username = message.username;
    }
    if (message.password !== "") {
      obj.password = message.password;
    }
    if (message.avatarId !== undefined) {
      obj.avatarId = message.avatarId;
    }
    return obj;
  },

  create(base?: DeepPartial<UserCreateRequest>): UserCreateRequest {
    return UserCreateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<UserCreateRequest>): UserCreateRequest {
    const message = createBaseUserCreateRequest();
    message.username = object.username ?? "";
    message.password = object.password ?? "";
    message.avatarId = object.avatarId ?? undefined;
    return message;
  },
};

function createBaseUserUpdateRequest(): UserUpdateRequest {
  return { userId: "", password: undefined, avatarId: undefined };
}

export const UserUpdateRequest = {
  encode(message: UserUpdateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userId !== "") {
      writer.uint32(10).string(message.userId);
    }
    if (message.password !== undefined) {
      writer.uint32(18).string(message.password);
    }
    if (message.avatarId !== undefined) {
      writer.uint32(26).string(message.avatarId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserUpdateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUserUpdateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.userId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.password = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.avatarId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): UserUpdateRequest {
    return {
      userId: isSet(object.userId) ? globalThis.String(object.userId) : "",
      password: isSet(object.password) ? globalThis.String(object.password) : undefined,
      avatarId: isSet(object.avatarId) ? globalThis.String(object.avatarId) : undefined,
    };
  },

  toJSON(message: UserUpdateRequest): unknown {
    const obj: any = {};
    if (message.userId !== "") {
      obj.userId = message.userId;
    }
    if (message.password !== undefined) {
      obj.password = message.password;
    }
    if (message.avatarId !== undefined) {
      obj.avatarId = message.avatarId;
    }
    return obj;
  },

  create(base?: DeepPartial<UserUpdateRequest>): UserUpdateRequest {
    return UserUpdateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<UserUpdateRequest>): UserUpdateRequest {
    const message = createBaseUserUpdateRequest();
    message.userId = object.userId ?? "";
    message.password = object.password ?? undefined;
    message.avatarId = object.avatarId ?? undefined;
    return message;
  },
};

function createBaseUserDeleteRequest(): UserDeleteRequest {
  return { userId: "" };
}

export const UserDeleteRequest = {
  encode(message: UserDeleteRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userId !== "") {
      writer.uint32(10).string(message.userId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserDeleteRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUserDeleteRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.userId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): UserDeleteRequest {
    return { userId: isSet(object.userId) ? globalThis.String(object.userId) : "" };
  },

  toJSON(message: UserDeleteRequest): unknown {
    const obj: any = {};
    if (message.userId !== "") {
      obj.userId = message.userId;
    }
    return obj;
  },

  create(base?: DeepPartial<UserDeleteRequest>): UserDeleteRequest {
    return UserDeleteRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<UserDeleteRequest>): UserDeleteRequest {
    const message = createBaseUserDeleteRequest();
    message.userId = object.userId ?? "";
    return message;
  },
};

function createBaseImageCreateRequest(): ImageCreateRequest {
  return { content: new Uint8Array(0) };
}

export const ImageCreateRequest = {
  encode(message: ImageCreateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.content.length !== 0) {
      writer.uint32(10).bytes(message.content);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ImageCreateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseImageCreateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.content = reader.bytes();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ImageCreateRequest {
    return { content: isSet(object.content) ? bytesFromBase64(object.content) : new Uint8Array(0) };
  },

  toJSON(message: ImageCreateRequest): unknown {
    const obj: any = {};
    if (message.content.length !== 0) {
      obj.content = base64FromBytes(message.content);
    }
    return obj;
  },

  create(base?: DeepPartial<ImageCreateRequest>): ImageCreateRequest {
    return ImageCreateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ImageCreateRequest>): ImageCreateRequest {
    const message = createBaseImageCreateRequest();
    message.content = object.content ?? new Uint8Array(0);
    return message;
  },
};

function createBaseImageCreateResponse(): ImageCreateResponse {
  return { imageId: "" };
}

export const ImageCreateResponse = {
  encode(message: ImageCreateResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.imageId !== "") {
      writer.uint32(10).string(message.imageId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ImageCreateResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseImageCreateResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.imageId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ImageCreateResponse {
    return { imageId: isSet(object.imageId) ? globalThis.String(object.imageId) : "" };
  },

  toJSON(message: ImageCreateResponse): unknown {
    const obj: any = {};
    if (message.imageId !== "") {
      obj.imageId = message.imageId;
    }
    return obj;
  },

  create(base?: DeepPartial<ImageCreateResponse>): ImageCreateResponse {
    return ImageCreateResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ImageCreateResponse>): ImageCreateResponse {
    const message = createBaseImageCreateResponse();
    message.imageId = object.imageId ?? "";
    return message;
  },
};

function createBaseImageDeleteRequest(): ImageDeleteRequest {
  return { imageId: "" };
}

export const ImageDeleteRequest = {
  encode(message: ImageDeleteRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.imageId !== "") {
      writer.uint32(10).string(message.imageId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ImageDeleteRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseImageDeleteRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.imageId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ImageDeleteRequest {
    return { imageId: isSet(object.imageId) ? globalThis.String(object.imageId) : "" };
  },

  toJSON(message: ImageDeleteRequest): unknown {
    const obj: any = {};
    if (message.imageId !== "") {
      obj.imageId = message.imageId;
    }
    return obj;
  },

  create(base?: DeepPartial<ImageDeleteRequest>): ImageDeleteRequest {
    return ImageDeleteRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ImageDeleteRequest>): ImageDeleteRequest {
    const message = createBaseImageDeleteRequest();
    message.imageId = object.imageId ?? "";
    return message;
  },
};

function createBaseImageDownloadRequest(): ImageDownloadRequest {
  return { imageId: "" };
}

export const ImageDownloadRequest = {
  encode(message: ImageDownloadRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.imageId !== "") {
      writer.uint32(10).string(message.imageId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ImageDownloadRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseImageDownloadRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.imageId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ImageDownloadRequest {
    return { imageId: isSet(object.imageId) ? globalThis.String(object.imageId) : "" };
  },

  toJSON(message: ImageDownloadRequest): unknown {
    const obj: any = {};
    if (message.imageId !== "") {
      obj.imageId = message.imageId;
    }
    return obj;
  },

  create(base?: DeepPartial<ImageDownloadRequest>): ImageDownloadRequest {
    return ImageDownloadRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ImageDownloadRequest>): ImageDownloadRequest {
    const message = createBaseImageDownloadRequest();
    message.imageId = object.imageId ?? "";
    return message;
  },
};

function createBaseImageDownloadResponse(): ImageDownloadResponse {
  return { imageId: "", mimeType: "", content: new Uint8Array(0) };
}

export const ImageDownloadResponse = {
  encode(message: ImageDownloadResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.imageId !== "") {
      writer.uint32(10).string(message.imageId);
    }
    if (message.mimeType !== "") {
      writer.uint32(18).string(message.mimeType);
    }
    if (message.content.length !== 0) {
      writer.uint32(26).bytes(message.content);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ImageDownloadResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseImageDownloadResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.imageId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.mimeType = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.content = reader.bytes();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ImageDownloadResponse {
    return {
      imageId: isSet(object.imageId) ? globalThis.String(object.imageId) : "",
      mimeType: isSet(object.mimeType) ? globalThis.String(object.mimeType) : "",
      content: isSet(object.content) ? bytesFromBase64(object.content) : new Uint8Array(0),
    };
  },

  toJSON(message: ImageDownloadResponse): unknown {
    const obj: any = {};
    if (message.imageId !== "") {
      obj.imageId = message.imageId;
    }
    if (message.mimeType !== "") {
      obj.mimeType = message.mimeType;
    }
    if (message.content.length !== 0) {
      obj.content = base64FromBytes(message.content);
    }
    return obj;
  },

  create(base?: DeepPartial<ImageDownloadResponse>): ImageDownloadResponse {
    return ImageDownloadResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ImageDownloadResponse>): ImageDownloadResponse {
    const message = createBaseImageDownloadResponse();
    message.imageId = object.imageId ?? "";
    message.mimeType = object.mimeType ?? "";
    message.content = object.content ?? new Uint8Array(0);
    return message;
  },
};

function createBaseArtist(): Artist {
  return { id: "", name: "", albumCount: 0, listenCount: 0, coverartId: undefined, properties: [] };
}

export const Artist = {
  encode(message: Artist, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.name !== "") {
      writer.uint32(18).string(message.name);
    }
    if (message.albumCount !== 0) {
      writer.uint32(24).uint32(message.albumCount);
    }
    if (message.listenCount !== 0) {
      writer.uint32(32).uint32(message.listenCount);
    }
    if (message.coverartId !== undefined) {
      writer.uint32(42).string(message.coverartId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(50).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Artist {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtist();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.id = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 24) {
            break;
          }

          message.albumCount = reader.uint32();
          continue;
        case 4:
          if (tag !== 32) {
            break;
          }

          message.listenCount = reader.uint32();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 6:
          if (tag !== 50) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Artist {
    return {
      id: isSet(object.id) ? globalThis.String(object.id) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      albumCount: isSet(object.albumCount) ? globalThis.Number(object.albumCount) : 0,
      listenCount: isSet(object.listenCount) ? globalThis.Number(object.listenCount) : 0,
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: Artist): unknown {
    const obj: any = {};
    if (message.id !== "") {
      obj.id = message.id;
    }
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.albumCount !== 0) {
      obj.albumCount = Math.round(message.albumCount);
    }
    if (message.listenCount !== 0) {
      obj.listenCount = Math.round(message.listenCount);
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<Artist>): Artist {
    return Artist.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<Artist>): Artist {
    const message = createBaseArtist();
    message.id = object.id ?? "";
    message.name = object.name ?? "";
    message.albumCount = object.albumCount ?? 0;
    message.listenCount = object.listenCount ?? 0;
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseArtistListRequest(): ArtistListRequest {
  return { offset: undefined, count: undefined };
}

export const ArtistListRequest = {
  encode(message: ArtistListRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ArtistListRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtistListRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ArtistListRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
    };
  },

  toJSON(message: ArtistListRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    return obj;
  },

  create(base?: DeepPartial<ArtistListRequest>): ArtistListRequest {
    return ArtistListRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ArtistListRequest>): ArtistListRequest {
    const message = createBaseArtistListRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    return message;
  },
};

function createBaseArtistListResponse(): ArtistListResponse {
  return { artists: [] };
}

export const ArtistListResponse = {
  encode(message: ArtistListResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.artists) {
      Artist.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ArtistListResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtistListResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.artists.push(Artist.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ArtistListResponse {
    return {
      artists: globalThis.Array.isArray(object?.artists) ? object.artists.map((e: any) => Artist.fromJSON(e)) : [],
    };
  },

  toJSON(message: ArtistListResponse): unknown {
    const obj: any = {};
    if (message.artists?.length) {
      obj.artists = message.artists.map((e) => Artist.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<ArtistListResponse>): ArtistListResponse {
    return ArtistListResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ArtistListResponse>): ArtistListResponse {
    const message = createBaseArtistListResponse();
    message.artists = object.artists?.map((e) => Artist.fromPartial(e)) || [];
    return message;
  },
};

function createBaseArtistGetRequest(): ArtistGetRequest {
  return { artistId: "" };
}

export const ArtistGetRequest = {
  encode(message: ArtistGetRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.artistId !== "") {
      writer.uint32(10).string(message.artistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ArtistGetRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtistGetRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.artistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ArtistGetRequest {
    return { artistId: isSet(object.artistId) ? globalThis.String(object.artistId) : "" };
  },

  toJSON(message: ArtistGetRequest): unknown {
    const obj: any = {};
    if (message.artistId !== "") {
      obj.artistId = message.artistId;
    }
    return obj;
  },

  create(base?: DeepPartial<ArtistGetRequest>): ArtistGetRequest {
    return ArtistGetRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ArtistGetRequest>): ArtistGetRequest {
    const message = createBaseArtistGetRequest();
    message.artistId = object.artistId ?? "";
    return message;
  },
};

function createBaseArtistCreateRequest(): ArtistCreateRequest {
  return { name: "", coverartId: undefined, properties: [] };
}

export const ArtistCreateRequest = {
  encode(message: ArtistCreateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.coverartId !== undefined) {
      writer.uint32(18).string(message.coverartId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ArtistCreateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtistCreateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.name = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ArtistCreateRequest {
    return {
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: ArtistCreateRequest): unknown {
    const obj: any = {};
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<ArtistCreateRequest>): ArtistCreateRequest {
    return ArtistCreateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ArtistCreateRequest>): ArtistCreateRequest {
    const message = createBaseArtistCreateRequest();
    message.name = object.name ?? "";
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseArtistDeleteRequest(): ArtistDeleteRequest {
  return { artistId: "" };
}

export const ArtistDeleteRequest = {
  encode(message: ArtistDeleteRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.artistId !== "") {
      writer.uint32(10).string(message.artistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ArtistDeleteRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtistDeleteRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.artistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ArtistDeleteRequest {
    return { artistId: isSet(object.artistId) ? globalThis.String(object.artistId) : "" };
  },

  toJSON(message: ArtistDeleteRequest): unknown {
    const obj: any = {};
    if (message.artistId !== "") {
      obj.artistId = message.artistId;
    }
    return obj;
  },

  create(base?: DeepPartial<ArtistDeleteRequest>): ArtistDeleteRequest {
    return ArtistDeleteRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ArtistDeleteRequest>): ArtistDeleteRequest {
    const message = createBaseArtistDeleteRequest();
    message.artistId = object.artistId ?? "";
    return message;
  },
};

function createBaseArtistUpdateRequest(): ArtistUpdateRequest {
  return { artistId: "", name: undefined, properties: [], mask: undefined };
}

export const ArtistUpdateRequest = {
  encode(message: ArtistUpdateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.artistId !== "") {
      writer.uint32(10).string(message.artistId);
    }
    if (message.name !== undefined) {
      writer.uint32(18).string(message.name);
    }
    for (const v of message.properties) {
      PropertyUpdate.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    if (message.mask !== undefined) {
      FieldMask.encode(FieldMask.wrap(message.mask), writer.uint32(34).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ArtistUpdateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseArtistUpdateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.artistId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.properties.push(PropertyUpdate.decode(reader, reader.uint32()));
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.mask = FieldMask.unwrap(FieldMask.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ArtistUpdateRequest {
    return {
      artistId: isSet(object.artistId) ? globalThis.String(object.artistId) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => PropertyUpdate.fromJSON(e))
        : [],
      mask: isSet(object.mask) ? FieldMask.unwrap(FieldMask.fromJSON(object.mask)) : undefined,
    };
  },

  toJSON(message: ArtistUpdateRequest): unknown {
    const obj: any = {};
    if (message.artistId !== "") {
      obj.artistId = message.artistId;
    }
    if (message.name !== undefined) {
      obj.name = message.name;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => PropertyUpdate.toJSON(e));
    }
    if (message.mask !== undefined) {
      obj.mask = FieldMask.toJSON(FieldMask.wrap(message.mask));
    }
    return obj;
  },

  create(base?: DeepPartial<ArtistUpdateRequest>): ArtistUpdateRequest {
    return ArtistUpdateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ArtistUpdateRequest>): ArtistUpdateRequest {
    const message = createBaseArtistUpdateRequest();
    message.artistId = object.artistId ?? "";
    message.name = object.name ?? undefined;
    message.properties = object.properties?.map((e) => PropertyUpdate.fromPartial(e)) || [];
    message.mask = object.mask ?? undefined;
    return message;
  },
};

function createBaseAlbum(): Album {
  return {
    id: "",
    name: "",
    trackCount: 0,
    duration: undefined,
    listenCount: 0,
    artists: [],
    coverartId: undefined,
    properties: [],
  };
}

export const Album = {
  encode(message: Album, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.name !== "") {
      writer.uint32(18).string(message.name);
    }
    if (message.trackCount !== 0) {
      writer.uint32(24).uint32(message.trackCount);
    }
    if (message.duration !== undefined) {
      Duration.encode(message.duration, writer.uint32(34).fork()).ldelim();
    }
    if (message.listenCount !== 0) {
      writer.uint32(40).uint32(message.listenCount);
    }
    writer.uint32(50).fork();
    for (const v of message.artists) {
      writer.uint32(v);
    }
    writer.ldelim();
    if (message.coverartId !== undefined) {
      writer.uint32(58).string(message.coverartId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(66).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Album {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbum();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.id = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 24) {
            break;
          }

          message.trackCount = reader.uint32();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.duration = Duration.decode(reader, reader.uint32());
          continue;
        case 5:
          if (tag !== 40) {
            break;
          }

          message.listenCount = reader.uint32();
          continue;
        case 6:
          if (tag === 48) {
            message.artists.push(reader.uint32());

            continue;
          }

          if (tag === 50) {
            const end2 = reader.uint32() + reader.pos;
            while (reader.pos < end2) {
              message.artists.push(reader.uint32());
            }

            continue;
          }

          break;
        case 7:
          if (tag !== 58) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 8:
          if (tag !== 66) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Album {
    return {
      id: isSet(object.id) ? globalThis.String(object.id) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      trackCount: isSet(object.trackCount) ? globalThis.Number(object.trackCount) : 0,
      duration: isSet(object.duration) ? Duration.fromJSON(object.duration) : undefined,
      listenCount: isSet(object.listenCount) ? globalThis.Number(object.listenCount) : 0,
      artists: globalThis.Array.isArray(object?.artists) ? object.artists.map((e: any) => globalThis.Number(e)) : [],
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: Album): unknown {
    const obj: any = {};
    if (message.id !== "") {
      obj.id = message.id;
    }
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.trackCount !== 0) {
      obj.trackCount = Math.round(message.trackCount);
    }
    if (message.duration !== undefined) {
      obj.duration = Duration.toJSON(message.duration);
    }
    if (message.listenCount !== 0) {
      obj.listenCount = Math.round(message.listenCount);
    }
    if (message.artists?.length) {
      obj.artists = message.artists.map((e) => Math.round(e));
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<Album>): Album {
    return Album.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<Album>): Album {
    const message = createBaseAlbum();
    message.id = object.id ?? "";
    message.name = object.name ?? "";
    message.trackCount = object.trackCount ?? 0;
    message.duration = (object.duration !== undefined && object.duration !== null)
      ? Duration.fromPartial(object.duration)
      : undefined;
    message.listenCount = object.listenCount ?? 0;
    message.artists = object.artists?.map((e) => e) || [];
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseAlbumListRequest(): AlbumListRequest {
  return { offset: undefined, count: undefined };
}

export const AlbumListRequest = {
  encode(message: AlbumListRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumListRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumListRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumListRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
    };
  },

  toJSON(message: AlbumListRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumListRequest>): AlbumListRequest {
    return AlbumListRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumListRequest>): AlbumListRequest {
    const message = createBaseAlbumListRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    return message;
  },
};

function createBaseAlbumListByArtistRequest(): AlbumListByArtistRequest {
  return { offset: undefined, count: undefined, artistId: "" };
}

export const AlbumListByArtistRequest = {
  encode(message: AlbumListByArtistRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    if (message.artistId !== "") {
      writer.uint32(26).string(message.artistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumListByArtistRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumListByArtistRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.artistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumListByArtistRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
      artistId: isSet(object.artistId) ? globalThis.String(object.artistId) : "",
    };
  },

  toJSON(message: AlbumListByArtistRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    if (message.artistId !== "") {
      obj.artistId = message.artistId;
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumListByArtistRequest>): AlbumListByArtistRequest {
    return AlbumListByArtistRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumListByArtistRequest>): AlbumListByArtistRequest {
    const message = createBaseAlbumListByArtistRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    message.artistId = object.artistId ?? "";
    return message;
  },
};

function createBaseAlbumListResponse(): AlbumListResponse {
  return { albums: [] };
}

export const AlbumListResponse = {
  encode(message: AlbumListResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.albums) {
      Album.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumListResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumListResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.albums.push(Album.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumListResponse {
    return { albums: globalThis.Array.isArray(object?.albums) ? object.albums.map((e: any) => Album.fromJSON(e)) : [] };
  },

  toJSON(message: AlbumListResponse): unknown {
    const obj: any = {};
    if (message.albums?.length) {
      obj.albums = message.albums.map((e) => Album.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumListResponse>): AlbumListResponse {
    return AlbumListResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumListResponse>): AlbumListResponse {
    const message = createBaseAlbumListResponse();
    message.albums = object.albums?.map((e) => Album.fromPartial(e)) || [];
    return message;
  },
};

function createBaseAlbumGetRequest(): AlbumGetRequest {
  return { albumId: "" };
}

export const AlbumGetRequest = {
  encode(message: AlbumGetRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.albumId !== "") {
      writer.uint32(10).string(message.albumId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumGetRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumGetRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.albumId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumGetRequest {
    return { albumId: isSet(object.albumId) ? globalThis.String(object.albumId) : "" };
  },

  toJSON(message: AlbumGetRequest): unknown {
    const obj: any = {};
    if (message.albumId !== "") {
      obj.albumId = message.albumId;
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumGetRequest>): AlbumGetRequest {
    return AlbumGetRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumGetRequest>): AlbumGetRequest {
    const message = createBaseAlbumGetRequest();
    message.albumId = object.albumId ?? "";
    return message;
  },
};

function createBaseAlbumCreateRequest(): AlbumCreateRequest {
  return { name: "", artistId: "", coverartId: undefined, properties: [] };
}

export const AlbumCreateRequest = {
  encode(message: AlbumCreateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.artistId !== "") {
      writer.uint32(18).string(message.artistId);
    }
    if (message.coverartId !== undefined) {
      writer.uint32(26).string(message.coverartId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumCreateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumCreateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.name = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.artistId = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumCreateRequest {
    return {
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      artistId: isSet(object.artistId) ? globalThis.String(object.artistId) : "",
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: AlbumCreateRequest): unknown {
    const obj: any = {};
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.artistId !== "") {
      obj.artistId = message.artistId;
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumCreateRequest>): AlbumCreateRequest {
    return AlbumCreateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumCreateRequest>): AlbumCreateRequest {
    const message = createBaseAlbumCreateRequest();
    message.name = object.name ?? "";
    message.artistId = object.artistId ?? "";
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseAlbumUpdateRequest(): AlbumUpdateRequest {
  return { albumId: "", name: undefined, artistId: undefined, coverartId: undefined, properties: [] };
}

export const AlbumUpdateRequest = {
  encode(message: AlbumUpdateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.albumId !== "") {
      writer.uint32(10).string(message.albumId);
    }
    if (message.name !== undefined) {
      writer.uint32(18).string(message.name);
    }
    if (message.artistId !== undefined) {
      writer.uint32(26).string(message.artistId);
    }
    if (message.coverartId !== undefined) {
      writer.uint32(34).string(message.coverartId);
    }
    for (const v of message.properties) {
      PropertyUpdate.encode(v!, writer.uint32(42).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumUpdateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumUpdateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.albumId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.artistId = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.properties.push(PropertyUpdate.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumUpdateRequest {
    return {
      albumId: isSet(object.albumId) ? globalThis.String(object.albumId) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : undefined,
      artistId: isSet(object.artistId) ? globalThis.String(object.artistId) : undefined,
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => PropertyUpdate.fromJSON(e))
        : [],
    };
  },

  toJSON(message: AlbumUpdateRequest): unknown {
    const obj: any = {};
    if (message.albumId !== "") {
      obj.albumId = message.albumId;
    }
    if (message.name !== undefined) {
      obj.name = message.name;
    }
    if (message.artistId !== undefined) {
      obj.artistId = message.artistId;
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => PropertyUpdate.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumUpdateRequest>): AlbumUpdateRequest {
    return AlbumUpdateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumUpdateRequest>): AlbumUpdateRequest {
    const message = createBaseAlbumUpdateRequest();
    message.albumId = object.albumId ?? "";
    message.name = object.name ?? undefined;
    message.artistId = object.artistId ?? undefined;
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => PropertyUpdate.fromPartial(e)) || [];
    return message;
  },
};

function createBaseAlbumDeleteRequest(): AlbumDeleteRequest {
  return { albumId: "" };
}

export const AlbumDeleteRequest = {
  encode(message: AlbumDeleteRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.albumId !== "") {
      writer.uint32(10).string(message.albumId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AlbumDeleteRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseAlbumDeleteRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.albumId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): AlbumDeleteRequest {
    return { albumId: isSet(object.albumId) ? globalThis.String(object.albumId) : "" };
  },

  toJSON(message: AlbumDeleteRequest): unknown {
    const obj: any = {};
    if (message.albumId !== "") {
      obj.albumId = message.albumId;
    }
    return obj;
  },

  create(base?: DeepPartial<AlbumDeleteRequest>): AlbumDeleteRequest {
    return AlbumDeleteRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<AlbumDeleteRequest>): AlbumDeleteRequest {
    const message = createBaseAlbumDeleteRequest();
    message.albumId = object.albumId ?? "";
    return message;
  },
};

function createBaseTrack(): Track {
  return {
    id: "",
    name: "",
    artistId: 0,
    albumId: 0,
    duration: undefined,
    listenCount: 0,
    coverArtId: undefined,
    properties: [],
  };
}

export const Track = {
  encode(message: Track, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.name !== "") {
      writer.uint32(18).string(message.name);
    }
    if (message.artistId !== 0) {
      writer.uint32(24).uint32(message.artistId);
    }
    if (message.albumId !== 0) {
      writer.uint32(32).uint32(message.albumId);
    }
    if (message.duration !== undefined) {
      Duration.encode(message.duration, writer.uint32(42).fork()).ldelim();
    }
    if (message.listenCount !== 0) {
      writer.uint32(48).uint32(message.listenCount);
    }
    if (message.coverArtId !== undefined) {
      writer.uint32(58).string(message.coverArtId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(66).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Track {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrack();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.id = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 24) {
            break;
          }

          message.artistId = reader.uint32();
          continue;
        case 4:
          if (tag !== 32) {
            break;
          }

          message.albumId = reader.uint32();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.duration = Duration.decode(reader, reader.uint32());
          continue;
        case 6:
          if (tag !== 48) {
            break;
          }

          message.listenCount = reader.uint32();
          continue;
        case 7:
          if (tag !== 58) {
            break;
          }

          message.coverArtId = reader.string();
          continue;
        case 8:
          if (tag !== 66) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Track {
    return {
      id: isSet(object.id) ? globalThis.String(object.id) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      artistId: isSet(object.artistId) ? globalThis.Number(object.artistId) : 0,
      albumId: isSet(object.albumId) ? globalThis.Number(object.albumId) : 0,
      duration: isSet(object.duration) ? Duration.fromJSON(object.duration) : undefined,
      listenCount: isSet(object.listenCount) ? globalThis.Number(object.listenCount) : 0,
      coverArtId: isSet(object.coverArtId) ? globalThis.String(object.coverArtId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: Track): unknown {
    const obj: any = {};
    if (message.id !== "") {
      obj.id = message.id;
    }
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.artistId !== 0) {
      obj.artistId = Math.round(message.artistId);
    }
    if (message.albumId !== 0) {
      obj.albumId = Math.round(message.albumId);
    }
    if (message.duration !== undefined) {
      obj.duration = Duration.toJSON(message.duration);
    }
    if (message.listenCount !== 0) {
      obj.listenCount = Math.round(message.listenCount);
    }
    if (message.coverArtId !== undefined) {
      obj.coverArtId = message.coverArtId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<Track>): Track {
    return Track.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<Track>): Track {
    const message = createBaseTrack();
    message.id = object.id ?? "";
    message.name = object.name ?? "";
    message.artistId = object.artistId ?? 0;
    message.albumId = object.albumId ?? 0;
    message.duration = (object.duration !== undefined && object.duration !== null)
      ? Duration.fromPartial(object.duration)
      : undefined;
    message.listenCount = object.listenCount ?? 0;
    message.coverArtId = object.coverArtId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseTrackListRequest(): TrackListRequest {
  return { offset: undefined, count: undefined };
}

export const TrackListRequest = {
  encode(message: TrackListRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackListRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackListRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackListRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
    };
  },

  toJSON(message: TrackListRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    return obj;
  },

  create(base?: DeepPartial<TrackListRequest>): TrackListRequest {
    return TrackListRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackListRequest>): TrackListRequest {
    const message = createBaseTrackListRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    return message;
  },
};

function createBaseTrackListByAlbumRequest(): TrackListByAlbumRequest {
  return { offset: undefined, count: undefined, albumId: "" };
}

export const TrackListByAlbumRequest = {
  encode(message: TrackListByAlbumRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    if (message.albumId !== "") {
      writer.uint32(26).string(message.albumId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackListByAlbumRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackListByAlbumRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.albumId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackListByAlbumRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
      albumId: isSet(object.albumId) ? globalThis.String(object.albumId) : "",
    };
  },

  toJSON(message: TrackListByAlbumRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    if (message.albumId !== "") {
      obj.albumId = message.albumId;
    }
    return obj;
  },

  create(base?: DeepPartial<TrackListByAlbumRequest>): TrackListByAlbumRequest {
    return TrackListByAlbumRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackListByAlbumRequest>): TrackListByAlbumRequest {
    const message = createBaseTrackListByAlbumRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    message.albumId = object.albumId ?? "";
    return message;
  },
};

function createBaseTrackListResponse(): TrackListResponse {
  return { tracks: [] };
}

export const TrackListResponse = {
  encode(message: TrackListResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.tracks) {
      Track.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackListResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackListResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.tracks.push(Track.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackListResponse {
    return { tracks: globalThis.Array.isArray(object?.tracks) ? object.tracks.map((e: any) => Track.fromJSON(e)) : [] };
  },

  toJSON(message: TrackListResponse): unknown {
    const obj: any = {};
    if (message.tracks?.length) {
      obj.tracks = message.tracks.map((e) => Track.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<TrackListResponse>): TrackListResponse {
    return TrackListResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackListResponse>): TrackListResponse {
    const message = createBaseTrackListResponse();
    message.tracks = object.tracks?.map((e) => Track.fromPartial(e)) || [];
    return message;
  },
};

function createBaseTrackGetRequest(): TrackGetRequest {
  return { trackId: "" };
}

export const TrackGetRequest = {
  encode(message: TrackGetRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.trackId !== "") {
      writer.uint32(10).string(message.trackId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackGetRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackGetRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.trackId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackGetRequest {
    return { trackId: isSet(object.trackId) ? globalThis.String(object.trackId) : "" };
  },

  toJSON(message: TrackGetRequest): unknown {
    const obj: any = {};
    if (message.trackId !== "") {
      obj.trackId = message.trackId;
    }
    return obj;
  },

  create(base?: DeepPartial<TrackGetRequest>): TrackGetRequest {
    return TrackGetRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackGetRequest>): TrackGetRequest {
    const message = createBaseTrackGetRequest();
    message.trackId = object.trackId ?? "";
    return message;
  },
};

function createBaseTrackCreateRequest(): TrackCreateRequest {
  return { name: "", albumId: "", coverartId: undefined, audioId: undefined, properties: [] };
}

export const TrackCreateRequest = {
  encode(message: TrackCreateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.albumId !== "") {
      writer.uint32(18).string(message.albumId);
    }
    if (message.coverartId !== undefined) {
      writer.uint32(26).string(message.coverartId);
    }
    if (message.audioId !== undefined) {
      writer.uint32(34).string(message.audioId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(42).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackCreateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackCreateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.name = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.albumId = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.audioId = reader.string();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackCreateRequest {
    return {
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      albumId: isSet(object.albumId) ? globalThis.String(object.albumId) : "",
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      audioId: isSet(object.audioId) ? globalThis.String(object.audioId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: TrackCreateRequest): unknown {
    const obj: any = {};
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.albumId !== "") {
      obj.albumId = message.albumId;
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.audioId !== undefined) {
      obj.audioId = message.audioId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<TrackCreateRequest>): TrackCreateRequest {
    return TrackCreateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackCreateRequest>): TrackCreateRequest {
    const message = createBaseTrackCreateRequest();
    message.name = object.name ?? "";
    message.albumId = object.albumId ?? "";
    message.coverartId = object.coverartId ?? undefined;
    message.audioId = object.audioId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseTrackUpdateRequest(): TrackUpdateRequest {
  return { trackId: "", name: undefined, albumId: undefined, coverartId: undefined, properties: [] };
}

export const TrackUpdateRequest = {
  encode(message: TrackUpdateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.trackId !== "") {
      writer.uint32(10).string(message.trackId);
    }
    if (message.name !== undefined) {
      writer.uint32(18).string(message.name);
    }
    if (message.albumId !== undefined) {
      writer.uint32(26).string(message.albumId);
    }
    if (message.coverartId !== undefined) {
      writer.uint32(34).string(message.coverartId);
    }
    for (const v of message.properties) {
      PropertyUpdate.encode(v!, writer.uint32(42).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackUpdateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackUpdateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.trackId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.albumId = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.properties.push(PropertyUpdate.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackUpdateRequest {
    return {
      trackId: isSet(object.trackId) ? globalThis.String(object.trackId) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : undefined,
      albumId: isSet(object.albumId) ? globalThis.String(object.albumId) : undefined,
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => PropertyUpdate.fromJSON(e))
        : [],
    };
  },

  toJSON(message: TrackUpdateRequest): unknown {
    const obj: any = {};
    if (message.trackId !== "") {
      obj.trackId = message.trackId;
    }
    if (message.name !== undefined) {
      obj.name = message.name;
    }
    if (message.albumId !== undefined) {
      obj.albumId = message.albumId;
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => PropertyUpdate.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<TrackUpdateRequest>): TrackUpdateRequest {
    return TrackUpdateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackUpdateRequest>): TrackUpdateRequest {
    const message = createBaseTrackUpdateRequest();
    message.trackId = object.trackId ?? "";
    message.name = object.name ?? undefined;
    message.albumId = object.albumId ?? undefined;
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => PropertyUpdate.fromPartial(e)) || [];
    return message;
  },
};

function createBaseTrackDeleteRequest(): TrackDeleteRequest {
  return { trackId: "" };
}

export const TrackDeleteRequest = {
  encode(message: TrackDeleteRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.trackId !== "") {
      writer.uint32(10).string(message.trackId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackDeleteRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackDeleteRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.trackId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackDeleteRequest {
    return { trackId: isSet(object.trackId) ? globalThis.String(object.trackId) : "" };
  },

  toJSON(message: TrackDeleteRequest): unknown {
    const obj: any = {};
    if (message.trackId !== "") {
      obj.trackId = message.trackId;
    }
    return obj;
  },

  create(base?: DeepPartial<TrackDeleteRequest>): TrackDeleteRequest {
    return TrackDeleteRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackDeleteRequest>): TrackDeleteRequest {
    const message = createBaseTrackDeleteRequest();
    message.trackId = object.trackId ?? "";
    return message;
  },
};

function createBasePlaylist(): Playlist {
  return { id: "", name: "", userId: "", trackCount: 0, duration: undefined, coverartId: undefined, properties: [] };
}

export const Playlist = {
  encode(message: Playlist, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.name !== "") {
      writer.uint32(18).string(message.name);
    }
    if (message.userId !== "") {
      writer.uint32(26).string(message.userId);
    }
    if (message.trackCount !== 0) {
      writer.uint32(32).uint32(message.trackCount);
    }
    if (message.duration !== undefined) {
      Duration.encode(message.duration, writer.uint32(42).fork()).ldelim();
    }
    if (message.coverartId !== undefined) {
      writer.uint32(50).string(message.coverartId);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(58).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Playlist {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylist();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.id = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.userId = reader.string();
          continue;
        case 4:
          if (tag !== 32) {
            break;
          }

          message.trackCount = reader.uint32();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.duration = Duration.decode(reader, reader.uint32());
          continue;
        case 6:
          if (tag !== 50) {
            break;
          }

          message.coverartId = reader.string();
          continue;
        case 7:
          if (tag !== 58) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Playlist {
    return {
      id: isSet(object.id) ? globalThis.String(object.id) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      userId: isSet(object.userId) ? globalThis.String(object.userId) : "",
      trackCount: isSet(object.trackCount) ? globalThis.Number(object.trackCount) : 0,
      duration: isSet(object.duration) ? Duration.fromJSON(object.duration) : undefined,
      coverartId: isSet(object.coverartId) ? globalThis.String(object.coverartId) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: Playlist): unknown {
    const obj: any = {};
    if (message.id !== "") {
      obj.id = message.id;
    }
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.userId !== "") {
      obj.userId = message.userId;
    }
    if (message.trackCount !== 0) {
      obj.trackCount = Math.round(message.trackCount);
    }
    if (message.duration !== undefined) {
      obj.duration = Duration.toJSON(message.duration);
    }
    if (message.coverartId !== undefined) {
      obj.coverartId = message.coverartId;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<Playlist>): Playlist {
    return Playlist.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<Playlist>): Playlist {
    const message = createBasePlaylist();
    message.id = object.id ?? "";
    message.name = object.name ?? "";
    message.userId = object.userId ?? "";
    message.trackCount = object.trackCount ?? 0;
    message.duration = (object.duration !== undefined && object.duration !== null)
      ? Duration.fromPartial(object.duration)
      : undefined;
    message.coverartId = object.coverartId ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBasePlaylistListRequest(): PlaylistListRequest {
  return { offset: undefined, count: undefined };
}

export const PlaylistListRequest = {
  encode(message: PlaylistListRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.offset !== undefined) {
      writer.uint32(8).uint32(message.offset);
    }
    if (message.count !== undefined) {
      writer.uint32(16).uint32(message.count);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistListRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistListRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.offset = reader.uint32();
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.count = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistListRequest {
    return {
      offset: isSet(object.offset) ? globalThis.Number(object.offset) : undefined,
      count: isSet(object.count) ? globalThis.Number(object.count) : undefined,
    };
  },

  toJSON(message: PlaylistListRequest): unknown {
    const obj: any = {};
    if (message.offset !== undefined) {
      obj.offset = Math.round(message.offset);
    }
    if (message.count !== undefined) {
      obj.count = Math.round(message.count);
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistListRequest>): PlaylistListRequest {
    return PlaylistListRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistListRequest>): PlaylistListRequest {
    const message = createBasePlaylistListRequest();
    message.offset = object.offset ?? undefined;
    message.count = object.count ?? undefined;
    return message;
  },
};

function createBasePlaylistListResponse(): PlaylistListResponse {
  return { playlists: [] };
}

export const PlaylistListResponse = {
  encode(message: PlaylistListResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.playlists) {
      Playlist.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistListResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistListResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlists.push(Playlist.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistListResponse {
    return {
      playlists: globalThis.Array.isArray(object?.playlists)
        ? object.playlists.map((e: any) => Playlist.fromJSON(e))
        : [],
    };
  },

  toJSON(message: PlaylistListResponse): unknown {
    const obj: any = {};
    if (message.playlists?.length) {
      obj.playlists = message.playlists.map((e) => Playlist.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistListResponse>): PlaylistListResponse {
    return PlaylistListResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistListResponse>): PlaylistListResponse {
    const message = createBasePlaylistListResponse();
    message.playlists = object.playlists?.map((e) => Playlist.fromPartial(e)) || [];
    return message;
  },
};

function createBasePlaylistGetRequest(): PlaylistGetRequest {
  return { playlistId: "" };
}

export const PlaylistGetRequest = {
  encode(message: PlaylistGetRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistGetRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistGetRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistGetRequest {
    return { playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "" };
  },

  toJSON(message: PlaylistGetRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistGetRequest>): PlaylistGetRequest {
    return PlaylistGetRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistGetRequest>): PlaylistGetRequest {
    const message = createBasePlaylistGetRequest();
    message.playlistId = object.playlistId ?? "";
    return message;
  },
};

function createBasePlaylistCreateRequest(): PlaylistCreateRequest {
  return { name: "", ownerId: "", trackIds: [], properties: [] };
}

export const PlaylistCreateRequest = {
  encode(message: PlaylistCreateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.name !== "") {
      writer.uint32(10).string(message.name);
    }
    if (message.ownerId !== "") {
      writer.uint32(18).string(message.ownerId);
    }
    for (const v of message.trackIds) {
      writer.uint32(26).string(v!);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistCreateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistCreateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.name = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.ownerId = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.trackIds.push(reader.string());
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistCreateRequest {
    return {
      name: isSet(object.name) ? globalThis.String(object.name) : "",
      ownerId: isSet(object.ownerId) ? globalThis.String(object.ownerId) : "",
      trackIds: globalThis.Array.isArray(object?.trackIds) ? object.trackIds.map((e: any) => globalThis.String(e)) : [],
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: PlaylistCreateRequest): unknown {
    const obj: any = {};
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.ownerId !== "") {
      obj.ownerId = message.ownerId;
    }
    if (message.trackIds?.length) {
      obj.trackIds = message.trackIds;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistCreateRequest>): PlaylistCreateRequest {
    return PlaylistCreateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistCreateRequest>): PlaylistCreateRequest {
    const message = createBasePlaylistCreateRequest();
    message.name = object.name ?? "";
    message.ownerId = object.ownerId ?? "";
    message.trackIds = object.trackIds?.map((e) => e) || [];
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBasePlaylistUpdateRequest(): PlaylistUpdateRequest {
  return { playlistId: "", name: undefined, properties: [] };
}

export const PlaylistUpdateRequest = {
  encode(message: PlaylistUpdateRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    if (message.name !== undefined) {
      writer.uint32(18).string(message.name);
    }
    for (const v of message.properties) {
      PropertyUpdate.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistUpdateRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistUpdateRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.properties.push(PropertyUpdate.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistUpdateRequest {
    return {
      playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "",
      name: isSet(object.name) ? globalThis.String(object.name) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => PropertyUpdate.fromJSON(e))
        : [],
    };
  },

  toJSON(message: PlaylistUpdateRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    if (message.name !== undefined) {
      obj.name = message.name;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => PropertyUpdate.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistUpdateRequest>): PlaylistUpdateRequest {
    return PlaylistUpdateRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistUpdateRequest>): PlaylistUpdateRequest {
    const message = createBasePlaylistUpdateRequest();
    message.playlistId = object.playlistId ?? "";
    message.name = object.name ?? undefined;
    message.properties = object.properties?.map((e) => PropertyUpdate.fromPartial(e)) || [];
    return message;
  },
};

function createBasePlaylistDeleteRequest(): PlaylistDeleteRequest {
  return { playlistId: "" };
}

export const PlaylistDeleteRequest = {
  encode(message: PlaylistDeleteRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistDeleteRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistDeleteRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistDeleteRequest {
    return { playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "" };
  },

  toJSON(message: PlaylistDeleteRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistDeleteRequest>): PlaylistDeleteRequest {
    return PlaylistDeleteRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistDeleteRequest>): PlaylistDeleteRequest {
    const message = createBasePlaylistDeleteRequest();
    message.playlistId = object.playlistId ?? "";
    return message;
  },
};

function createBasePlaylistTrackListRequest(): PlaylistTrackListRequest {
  return { playlistId: "" };
}

export const PlaylistTrackListRequest = {
  encode(message: PlaylistTrackListRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistTrackListRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistTrackListRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistTrackListRequest {
    return { playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "" };
  },

  toJSON(message: PlaylistTrackListRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistTrackListRequest>): PlaylistTrackListRequest {
    return PlaylistTrackListRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistTrackListRequest>): PlaylistTrackListRequest {
    const message = createBasePlaylistTrackListRequest();
    message.playlistId = object.playlistId ?? "";
    return message;
  },
};

function createBasePlaylistTrackListResponse(): PlaylistTrackListResponse {
  return { tracks: [] };
}

export const PlaylistTrackListResponse = {
  encode(message: PlaylistTrackListResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.tracks) {
      Track.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistTrackListResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistTrackListResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.tracks.push(Track.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistTrackListResponse {
    return { tracks: globalThis.Array.isArray(object?.tracks) ? object.tracks.map((e: any) => Track.fromJSON(e)) : [] };
  },

  toJSON(message: PlaylistTrackListResponse): unknown {
    const obj: any = {};
    if (message.tracks?.length) {
      obj.tracks = message.tracks.map((e) => Track.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistTrackListResponse>): PlaylistTrackListResponse {
    return PlaylistTrackListResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistTrackListResponse>): PlaylistTrackListResponse {
    const message = createBasePlaylistTrackListResponse();
    message.tracks = object.tracks?.map((e) => Track.fromPartial(e)) || [];
    return message;
  },
};

function createBasePlaylistTrackInsertRequest(): PlaylistTrackInsertRequest {
  return { playlistId: "", trackIds: [] };
}

export const PlaylistTrackInsertRequest = {
  encode(message: PlaylistTrackInsertRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    for (const v of message.trackIds) {
      writer.uint32(18).string(v!);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistTrackInsertRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistTrackInsertRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.trackIds.push(reader.string());
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistTrackInsertRequest {
    return {
      playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "",
      trackIds: globalThis.Array.isArray(object?.trackIds) ? object.trackIds.map((e: any) => globalThis.String(e)) : [],
    };
  },

  toJSON(message: PlaylistTrackInsertRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    if (message.trackIds?.length) {
      obj.trackIds = message.trackIds;
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistTrackInsertRequest>): PlaylistTrackInsertRequest {
    return PlaylistTrackInsertRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistTrackInsertRequest>): PlaylistTrackInsertRequest {
    const message = createBasePlaylistTrackInsertRequest();
    message.playlistId = object.playlistId ?? "";
    message.trackIds = object.trackIds?.map((e) => e) || [];
    return message;
  },
};

function createBasePlaylistTrackRemoveRequest(): PlaylistTrackRemoveRequest {
  return { playlistId: "", trackIds: [] };
}

export const PlaylistTrackRemoveRequest = {
  encode(message: PlaylistTrackRemoveRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    for (const v of message.trackIds) {
      writer.uint32(18).string(v!);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistTrackRemoveRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistTrackRemoveRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.trackIds.push(reader.string());
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistTrackRemoveRequest {
    return {
      playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "",
      trackIds: globalThis.Array.isArray(object?.trackIds) ? object.trackIds.map((e: any) => globalThis.String(e)) : [],
    };
  },

  toJSON(message: PlaylistTrackRemoveRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    if (message.trackIds?.length) {
      obj.trackIds = message.trackIds;
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistTrackRemoveRequest>): PlaylistTrackRemoveRequest {
    return PlaylistTrackRemoveRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistTrackRemoveRequest>): PlaylistTrackRemoveRequest {
    const message = createBasePlaylistTrackRemoveRequest();
    message.playlistId = object.playlistId ?? "";
    message.trackIds = object.trackIds?.map((e) => e) || [];
    return message;
  },
};

function createBasePlaylistTrackClearRequest(): PlaylistTrackClearRequest {
  return { playlistId: "" };
}

export const PlaylistTrackClearRequest = {
  encode(message: PlaylistTrackClearRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.playlistId !== "") {
      writer.uint32(10).string(message.playlistId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): PlaylistTrackClearRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePlaylistTrackClearRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.playlistId = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): PlaylistTrackClearRequest {
    return { playlistId: isSet(object.playlistId) ? globalThis.String(object.playlistId) : "" };
  },

  toJSON(message: PlaylistTrackClearRequest): unknown {
    const obj: any = {};
    if (message.playlistId !== "") {
      obj.playlistId = message.playlistId;
    }
    return obj;
  },

  create(base?: DeepPartial<PlaylistTrackClearRequest>): PlaylistTrackClearRequest {
    return PlaylistTrackClearRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<PlaylistTrackClearRequest>): PlaylistTrackClearRequest {
    const message = createBasePlaylistTrackClearRequest();
    message.playlistId = object.playlistId ?? "";
    return message;
  },
};

function createBaseScrobble(): Scrobble {
  return { id: "", trackId: "", userId: "", duration: undefined, timestamp: undefined, properties: [] };
}

export const Scrobble = {
  encode(message: Scrobble, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.trackId !== "") {
      writer.uint32(18).string(message.trackId);
    }
    if (message.userId !== "") {
      writer.uint32(26).string(message.userId);
    }
    if (message.duration !== undefined) {
      Duration.encode(message.duration, writer.uint32(34).fork()).ldelim();
    }
    if (message.timestamp !== undefined) {
      Timestamp.encode(toTimestamp(message.timestamp), writer.uint32(42).fork()).ldelim();
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(50).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Scrobble {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseScrobble();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.id = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.trackId = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.userId = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.duration = Duration.decode(reader, reader.uint32());
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.timestamp = fromTimestamp(Timestamp.decode(reader, reader.uint32()));
          continue;
        case 6:
          if (tag !== 50) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Scrobble {
    return {
      id: isSet(object.id) ? globalThis.String(object.id) : "",
      trackId: isSet(object.trackId) ? globalThis.String(object.trackId) : "",
      userId: isSet(object.userId) ? globalThis.String(object.userId) : "",
      duration: isSet(object.duration) ? Duration.fromJSON(object.duration) : undefined,
      timestamp: isSet(object.timestamp) ? fromJsonTimestamp(object.timestamp) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
    };
  },

  toJSON(message: Scrobble): unknown {
    const obj: any = {};
    if (message.id !== "") {
      obj.id = message.id;
    }
    if (message.trackId !== "") {
      obj.trackId = message.trackId;
    }
    if (message.userId !== "") {
      obj.userId = message.userId;
    }
    if (message.duration !== undefined) {
      obj.duration = Duration.toJSON(message.duration);
    }
    if (message.timestamp !== undefined) {
      obj.timestamp = message.timestamp.toISOString();
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    return obj;
  },

  create(base?: DeepPartial<Scrobble>): Scrobble {
    return Scrobble.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<Scrobble>): Scrobble {
    const message = createBaseScrobble();
    message.id = object.id ?? "";
    message.trackId = object.trackId ?? "";
    message.userId = object.userId ?? "";
    message.duration = (object.duration !== undefined && object.duration !== null)
      ? Duration.fromPartial(object.duration)
      : undefined;
    message.timestamp = object.timestamp ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    return message;
  },
};

function createBaseImportRequest(): ImportRequest {
  return { chunk: new Uint8Array(0), filepath: undefined, artistId: undefined, albumId: undefined };
}

export const ImportRequest = {
  encode(message: ImportRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.chunk.length !== 0) {
      writer.uint32(10).bytes(message.chunk);
    }
    if (message.filepath !== undefined) {
      writer.uint32(18).string(message.filepath);
    }
    if (message.artistId !== undefined) {
      writer.uint32(24).uint32(message.artistId);
    }
    if (message.albumId !== undefined) {
      writer.uint32(32).uint32(message.albumId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ImportRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseImportRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.chunk = reader.bytes();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.filepath = reader.string();
          continue;
        case 3:
          if (tag !== 24) {
            break;
          }

          message.artistId = reader.uint32();
          continue;
        case 4:
          if (tag !== 32) {
            break;
          }

          message.albumId = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ImportRequest {
    return {
      chunk: isSet(object.chunk) ? bytesFromBase64(object.chunk) : new Uint8Array(0),
      filepath: isSet(object.filepath) ? globalThis.String(object.filepath) : undefined,
      artistId: isSet(object.artistId) ? globalThis.Number(object.artistId) : undefined,
      albumId: isSet(object.albumId) ? globalThis.Number(object.albumId) : undefined,
    };
  },

  toJSON(message: ImportRequest): unknown {
    const obj: any = {};
    if (message.chunk.length !== 0) {
      obj.chunk = base64FromBytes(message.chunk);
    }
    if (message.filepath !== undefined) {
      obj.filepath = message.filepath;
    }
    if (message.artistId !== undefined) {
      obj.artistId = Math.round(message.artistId);
    }
    if (message.albumId !== undefined) {
      obj.albumId = Math.round(message.albumId);
    }
    return obj;
  },

  create(base?: DeepPartial<ImportRequest>): ImportRequest {
    return ImportRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<ImportRequest>): ImportRequest {
    const message = createBaseImportRequest();
    message.chunk = object.chunk ?? new Uint8Array(0);
    message.filepath = object.filepath ?? undefined;
    message.artistId = object.artistId ?? undefined;
    message.albumId = object.albumId ?? undefined;
    return message;
  },
};

function createBaseMetadataFetchRequest(): MetadataFetchRequest {
  return { kind: 0, itemId: 0 };
}

export const MetadataFetchRequest = {
  encode(message: MetadataFetchRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.kind !== 0) {
      writer.uint32(8).int32(message.kind);
    }
    if (message.itemId !== 0) {
      writer.uint32(16).uint32(message.itemId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MetadataFetchRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMetadataFetchRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.kind = reader.int32() as any;
          continue;
        case 2:
          if (tag !== 16) {
            break;
          }

          message.itemId = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): MetadataFetchRequest {
    return {
      kind: isSet(object.kind) ? metadataFetchKindFromJSON(object.kind) : 0,
      itemId: isSet(object.itemId) ? globalThis.Number(object.itemId) : 0,
    };
  },

  toJSON(message: MetadataFetchRequest): unknown {
    const obj: any = {};
    if (message.kind !== 0) {
      obj.kind = metadataFetchKindToJSON(message.kind);
    }
    if (message.itemId !== 0) {
      obj.itemId = Math.round(message.itemId);
    }
    return obj;
  },

  create(base?: DeepPartial<MetadataFetchRequest>): MetadataFetchRequest {
    return MetadataFetchRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<MetadataFetchRequest>): MetadataFetchRequest {
    const message = createBaseMetadataFetchRequest();
    message.kind = object.kind ?? 0;
    message.itemId = object.itemId ?? 0;
    return message;
  },
};

function createBaseTrackMetadata(): TrackMetadata {
  return { name: undefined, properties: [], cover: undefined };
}

export const TrackMetadata = {
  encode(message: TrackMetadata, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.name !== undefined) {
      writer.uint32(10).string(message.name);
    }
    for (const v of message.properties) {
      Property.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.cover !== undefined) {
      writer.uint32(26).bytes(message.cover);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TrackMetadata {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTrackMetadata();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.name = reader.string();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.properties.push(Property.decode(reader, reader.uint32()));
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.cover = reader.bytes();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TrackMetadata {
    return {
      name: isSet(object.name) ? globalThis.String(object.name) : undefined,
      properties: globalThis.Array.isArray(object?.properties)
        ? object.properties.map((e: any) => Property.fromJSON(e))
        : [],
      cover: isSet(object.cover) ? bytesFromBase64(object.cover) : undefined,
    };
  },

  toJSON(message: TrackMetadata): unknown {
    const obj: any = {};
    if (message.name !== undefined) {
      obj.name = message.name;
    }
    if (message.properties?.length) {
      obj.properties = message.properties.map((e) => Property.toJSON(e));
    }
    if (message.cover !== undefined) {
      obj.cover = base64FromBytes(message.cover);
    }
    return obj;
  },

  create(base?: DeepPartial<TrackMetadata>): TrackMetadata {
    return TrackMetadata.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<TrackMetadata>): TrackMetadata {
    const message = createBaseTrackMetadata();
    message.name = object.name ?? undefined;
    message.properties = object.properties?.map((e) => Property.fromPartial(e)) || [];
    message.cover = object.cover ?? undefined;
    return message;
  },
};

function createBaseMetadataAlbumTracksRequest(): MetadataAlbumTracksRequest {
  return { albumId: 0 };
}

export const MetadataAlbumTracksRequest = {
  encode(message: MetadataAlbumTracksRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.albumId !== 0) {
      writer.uint32(8).uint32(message.albumId);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MetadataAlbumTracksRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMetadataAlbumTracksRequest();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.albumId = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): MetadataAlbumTracksRequest {
    return { albumId: isSet(object.albumId) ? globalThis.Number(object.albumId) : 0 };
  },

  toJSON(message: MetadataAlbumTracksRequest): unknown {
    const obj: any = {};
    if (message.albumId !== 0) {
      obj.albumId = Math.round(message.albumId);
    }
    return obj;
  },

  create(base?: DeepPartial<MetadataAlbumTracksRequest>): MetadataAlbumTracksRequest {
    return MetadataAlbumTracksRequest.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<MetadataAlbumTracksRequest>): MetadataAlbumTracksRequest {
    const message = createBaseMetadataAlbumTracksRequest();
    message.albumId = object.albumId ?? 0;
    return message;
  },
};

function createBaseMetadataAlbumTracksResponse(): MetadataAlbumTracksResponse {
  return { tracks: {} };
}

export const MetadataAlbumTracksResponse = {
  encode(message: MetadataAlbumTracksResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    Object.entries(message.tracks).forEach(([key, value]) => {
      MetadataAlbumTracksResponse_TracksEntry.encode({ key: key as any, value }, writer.uint32(10).fork()).ldelim();
    });
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MetadataAlbumTracksResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMetadataAlbumTracksResponse();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          const entry1 = MetadataAlbumTracksResponse_TracksEntry.decode(reader, reader.uint32());
          if (entry1.value !== undefined) {
            message.tracks[entry1.key] = entry1.value;
          }
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): MetadataAlbumTracksResponse {
    return {
      tracks: isObject(object.tracks)
        ? Object.entries(object.tracks).reduce<{ [key: number]: TrackMetadata }>((acc, [key, value]) => {
          acc[globalThis.Number(key)] = TrackMetadata.fromJSON(value);
          return acc;
        }, {})
        : {},
    };
  },

  toJSON(message: MetadataAlbumTracksResponse): unknown {
    const obj: any = {};
    if (message.tracks) {
      const entries = Object.entries(message.tracks);
      if (entries.length > 0) {
        obj.tracks = {};
        entries.forEach(([k, v]) => {
          obj.tracks[k] = TrackMetadata.toJSON(v);
        });
      }
    }
    return obj;
  },

  create(base?: DeepPartial<MetadataAlbumTracksResponse>): MetadataAlbumTracksResponse {
    return MetadataAlbumTracksResponse.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<MetadataAlbumTracksResponse>): MetadataAlbumTracksResponse {
    const message = createBaseMetadataAlbumTracksResponse();
    message.tracks = Object.entries(object.tracks ?? {}).reduce<{ [key: number]: TrackMetadata }>(
      (acc, [key, value]) => {
        if (value !== undefined) {
          acc[globalThis.Number(key)] = TrackMetadata.fromPartial(value);
        }
        return acc;
      },
      {},
    );
    return message;
  },
};

function createBaseMetadataAlbumTracksResponse_TracksEntry(): MetadataAlbumTracksResponse_TracksEntry {
  return { key: 0, value: undefined };
}

export const MetadataAlbumTracksResponse_TracksEntry = {
  encode(message: MetadataAlbumTracksResponse_TracksEntry, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.key !== 0) {
      writer.uint32(8).uint32(message.key);
    }
    if (message.value !== undefined) {
      TrackMetadata.encode(message.value, writer.uint32(18).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): MetadataAlbumTracksResponse_TracksEntry {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMetadataAlbumTracksResponse_TracksEntry();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 8) {
            break;
          }

          message.key = reader.uint32();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.value = TrackMetadata.decode(reader, reader.uint32());
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): MetadataAlbumTracksResponse_TracksEntry {
    return {
      key: isSet(object.key) ? globalThis.Number(object.key) : 0,
      value: isSet(object.value) ? TrackMetadata.fromJSON(object.value) : undefined,
    };
  },

  toJSON(message: MetadataAlbumTracksResponse_TracksEntry): unknown {
    const obj: any = {};
    if (message.key !== 0) {
      obj.key = Math.round(message.key);
    }
    if (message.value !== undefined) {
      obj.value = TrackMetadata.toJSON(message.value);
    }
    return obj;
  },

  create(base?: DeepPartial<MetadataAlbumTracksResponse_TracksEntry>): MetadataAlbumTracksResponse_TracksEntry {
    return MetadataAlbumTracksResponse_TracksEntry.fromPartial(base ?? {});
  },
  fromPartial(object: DeepPartial<MetadataAlbumTracksResponse_TracksEntry>): MetadataAlbumTracksResponse_TracksEntry {
    const message = createBaseMetadataAlbumTracksResponse_TracksEntry();
    message.key = object.key ?? 0;
    message.value = (object.value !== undefined && object.value !== null)
      ? TrackMetadata.fromPartial(object.value)
      : undefined;
    return message;
  },
};

export type SonarServiceDefinition = typeof SonarServiceDefinition;
export const SonarServiceDefinition = {
  name: "SonarService",
  fullName: "sonar.SonarService",
  methods: {
    userList: {
      name: "UserList",
      requestType: UserListRequest,
      requestStream: false,
      responseType: UserListResponse,
      responseStream: false,
      options: {},
    },
    userCreate: {
      name: "UserCreate",
      requestType: UserCreateRequest,
      requestStream: false,
      responseType: User,
      responseStream: false,
      options: {},
    },
    userUpdate: {
      name: "UserUpdate",
      requestType: UserUpdateRequest,
      requestStream: false,
      responseType: User,
      responseStream: false,
      options: {},
    },
    userDelete: {
      name: "UserDelete",
      requestType: UserDeleteRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    imageCreate: {
      name: "ImageCreate",
      requestType: ImageCreateRequest,
      requestStream: false,
      responseType: ImageCreateResponse,
      responseStream: false,
      options: {},
    },
    imageDelete: {
      name: "ImageDelete",
      requestType: ImageDeleteRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    imageDownload: {
      name: "ImageDownload",
      requestType: ImageDownloadRequest,
      requestStream: false,
      responseType: ImageDownloadResponse,
      responseStream: true,
      options: {},
    },
    artistList: {
      name: "ArtistList",
      requestType: ArtistListRequest,
      requestStream: false,
      responseType: ArtistListResponse,
      responseStream: false,
      options: {},
    },
    artistGet: {
      name: "ArtistGet",
      requestType: ArtistGetRequest,
      requestStream: false,
      responseType: Artist,
      responseStream: false,
      options: {},
    },
    artistCreate: {
      name: "ArtistCreate",
      requestType: ArtistCreateRequest,
      requestStream: false,
      responseType: Artist,
      responseStream: false,
      options: {},
    },
    artistUpdate: {
      name: "ArtistUpdate",
      requestType: ArtistUpdateRequest,
      requestStream: false,
      responseType: Artist,
      responseStream: false,
      options: {},
    },
    artistDelete: {
      name: "ArtistDelete",
      requestType: ArtistDeleteRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    albumList: {
      name: "AlbumList",
      requestType: AlbumListRequest,
      requestStream: false,
      responseType: AlbumListResponse,
      responseStream: false,
      options: {},
    },
    albumListByArtist: {
      name: "AlbumListByArtist",
      requestType: AlbumListByArtistRequest,
      requestStream: false,
      responseType: AlbumListResponse,
      responseStream: false,
      options: {},
    },
    albumGet: {
      name: "AlbumGet",
      requestType: AlbumGetRequest,
      requestStream: false,
      responseType: Album,
      responseStream: false,
      options: {},
    },
    albumCreate: {
      name: "AlbumCreate",
      requestType: AlbumCreateRequest,
      requestStream: false,
      responseType: Album,
      responseStream: false,
      options: {},
    },
    albumUpdate: {
      name: "AlbumUpdate",
      requestType: AlbumUpdateRequest,
      requestStream: false,
      responseType: Album,
      responseStream: false,
      options: {},
    },
    albumDelete: {
      name: "AlbumDelete",
      requestType: AlbumDeleteRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    trackList: {
      name: "TrackList",
      requestType: TrackListRequest,
      requestStream: false,
      responseType: TrackListResponse,
      responseStream: false,
      options: {},
    },
    trackListByAlbum: {
      name: "TrackListByAlbum",
      requestType: TrackListByAlbumRequest,
      requestStream: false,
      responseType: TrackListResponse,
      responseStream: false,
      options: {},
    },
    trackGet: {
      name: "TrackGet",
      requestType: TrackGetRequest,
      requestStream: false,
      responseType: Track,
      responseStream: false,
      options: {},
    },
    trackCreate: {
      name: "TrackCreate",
      requestType: TrackCreateRequest,
      requestStream: false,
      responseType: Track,
      responseStream: false,
      options: {},
    },
    trackUpdate: {
      name: "TrackUpdate",
      requestType: TrackUpdateRequest,
      requestStream: false,
      responseType: Track,
      responseStream: false,
      options: {},
    },
    trackDelete: {
      name: "TrackDelete",
      requestType: TrackDeleteRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    playlistList: {
      name: "PlaylistList",
      requestType: PlaylistListRequest,
      requestStream: false,
      responseType: PlaylistListResponse,
      responseStream: false,
      options: {},
    },
    playlistGet: {
      name: "PlaylistGet",
      requestType: PlaylistGetRequest,
      requestStream: false,
      responseType: Playlist,
      responseStream: false,
      options: {},
    },
    playlistCreate: {
      name: "PlaylistCreate",
      requestType: PlaylistCreateRequest,
      requestStream: false,
      responseType: Playlist,
      responseStream: false,
      options: {},
    },
    playlistUpdate: {
      name: "PlaylistUpdate",
      requestType: PlaylistUpdateRequest,
      requestStream: false,
      responseType: Playlist,
      responseStream: false,
      options: {},
    },
    playlistDelete: {
      name: "PlaylistDelete",
      requestType: PlaylistDeleteRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    playlistTrackList: {
      name: "PlaylistTrackList",
      requestType: PlaylistTrackListRequest,
      requestStream: false,
      responseType: PlaylistTrackListResponse,
      responseStream: false,
      options: {},
    },
    playlistTrackInsert: {
      name: "PlaylistTrackInsert",
      requestType: PlaylistTrackInsertRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    playlistTrackRemove: {
      name: "PlaylistTrackRemove",
      requestType: PlaylistTrackRemoveRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    playlistTrackClear: {
      name: "PlaylistTrackClear",
      requestType: PlaylistTrackClearRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    import: {
      name: "Import",
      requestType: ImportRequest,
      requestStream: true,
      responseType: Track,
      responseStream: false,
      options: {},
    },
    metadataFetch: {
      name: "MetadataFetch",
      requestType: MetadataFetchRequest,
      requestStream: false,
      responseType: Empty,
      responseStream: false,
      options: {},
    },
    metadataAlbumTracks: {
      name: "MetadataAlbumTracks",
      requestType: MetadataAlbumTracksRequest,
      requestStream: false,
      responseType: MetadataAlbumTracksResponse,
      responseStream: false,
      options: {},
    },
  },
} as const;

export interface SonarServiceImplementation<CallContextExt = {}> {
  userList(request: UserListRequest, context: CallContext & CallContextExt): Promise<DeepPartial<UserListResponse>>;
  userCreate(request: UserCreateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<User>>;
  userUpdate(request: UserUpdateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<User>>;
  userDelete(request: UserDeleteRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  imageCreate(
    request: ImageCreateRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<ImageCreateResponse>>;
  imageDelete(request: ImageDeleteRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  imageDownload(
    request: ImageDownloadRequest,
    context: CallContext & CallContextExt,
  ): ServerStreamingMethodResult<DeepPartial<ImageDownloadResponse>>;
  artistList(
    request: ArtistListRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<ArtistListResponse>>;
  artistGet(request: ArtistGetRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Artist>>;
  artistCreate(request: ArtistCreateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Artist>>;
  artistUpdate(request: ArtistUpdateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Artist>>;
  artistDelete(request: ArtistDeleteRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  albumList(request: AlbumListRequest, context: CallContext & CallContextExt): Promise<DeepPartial<AlbumListResponse>>;
  albumListByArtist(
    request: AlbumListByArtistRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<AlbumListResponse>>;
  albumGet(request: AlbumGetRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Album>>;
  albumCreate(request: AlbumCreateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Album>>;
  albumUpdate(request: AlbumUpdateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Album>>;
  albumDelete(request: AlbumDeleteRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  trackList(request: TrackListRequest, context: CallContext & CallContextExt): Promise<DeepPartial<TrackListResponse>>;
  trackListByAlbum(
    request: TrackListByAlbumRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<TrackListResponse>>;
  trackGet(request: TrackGetRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Track>>;
  trackCreate(request: TrackCreateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Track>>;
  trackUpdate(request: TrackUpdateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Track>>;
  trackDelete(request: TrackDeleteRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  playlistList(
    request: PlaylistListRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<PlaylistListResponse>>;
  playlistGet(request: PlaylistGetRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Playlist>>;
  playlistCreate(request: PlaylistCreateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Playlist>>;
  playlistUpdate(request: PlaylistUpdateRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Playlist>>;
  playlistDelete(request: PlaylistDeleteRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  playlistTrackList(
    request: PlaylistTrackListRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<PlaylistTrackListResponse>>;
  playlistTrackInsert(
    request: PlaylistTrackInsertRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<Empty>>;
  playlistTrackRemove(
    request: PlaylistTrackRemoveRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<Empty>>;
  playlistTrackClear(
    request: PlaylistTrackClearRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<Empty>>;
  import(request: AsyncIterable<ImportRequest>, context: CallContext & CallContextExt): Promise<DeepPartial<Track>>;
  metadataFetch(request: MetadataFetchRequest, context: CallContext & CallContextExt): Promise<DeepPartial<Empty>>;
  metadataAlbumTracks(
    request: MetadataAlbumTracksRequest,
    context: CallContext & CallContextExt,
  ): Promise<DeepPartial<MetadataAlbumTracksResponse>>;
}

export interface SonarServiceClient<CallOptionsExt = {}> {
  userList(request: DeepPartial<UserListRequest>, options?: CallOptions & CallOptionsExt): Promise<UserListResponse>;
  userCreate(request: DeepPartial<UserCreateRequest>, options?: CallOptions & CallOptionsExt): Promise<User>;
  userUpdate(request: DeepPartial<UserUpdateRequest>, options?: CallOptions & CallOptionsExt): Promise<User>;
  userDelete(request: DeepPartial<UserDeleteRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  imageCreate(
    request: DeepPartial<ImageCreateRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<ImageCreateResponse>;
  imageDelete(request: DeepPartial<ImageDeleteRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  imageDownload(
    request: DeepPartial<ImageDownloadRequest>,
    options?: CallOptions & CallOptionsExt,
  ): AsyncIterable<ImageDownloadResponse>;
  artistList(
    request: DeepPartial<ArtistListRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<ArtistListResponse>;
  artistGet(request: DeepPartial<ArtistGetRequest>, options?: CallOptions & CallOptionsExt): Promise<Artist>;
  artistCreate(request: DeepPartial<ArtistCreateRequest>, options?: CallOptions & CallOptionsExt): Promise<Artist>;
  artistUpdate(request: DeepPartial<ArtistUpdateRequest>, options?: CallOptions & CallOptionsExt): Promise<Artist>;
  artistDelete(request: DeepPartial<ArtistDeleteRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  albumList(request: DeepPartial<AlbumListRequest>, options?: CallOptions & CallOptionsExt): Promise<AlbumListResponse>;
  albumListByArtist(
    request: DeepPartial<AlbumListByArtistRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<AlbumListResponse>;
  albumGet(request: DeepPartial<AlbumGetRequest>, options?: CallOptions & CallOptionsExt): Promise<Album>;
  albumCreate(request: DeepPartial<AlbumCreateRequest>, options?: CallOptions & CallOptionsExt): Promise<Album>;
  albumUpdate(request: DeepPartial<AlbumUpdateRequest>, options?: CallOptions & CallOptionsExt): Promise<Album>;
  albumDelete(request: DeepPartial<AlbumDeleteRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  trackList(request: DeepPartial<TrackListRequest>, options?: CallOptions & CallOptionsExt): Promise<TrackListResponse>;
  trackListByAlbum(
    request: DeepPartial<TrackListByAlbumRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<TrackListResponse>;
  trackGet(request: DeepPartial<TrackGetRequest>, options?: CallOptions & CallOptionsExt): Promise<Track>;
  trackCreate(request: DeepPartial<TrackCreateRequest>, options?: CallOptions & CallOptionsExt): Promise<Track>;
  trackUpdate(request: DeepPartial<TrackUpdateRequest>, options?: CallOptions & CallOptionsExt): Promise<Track>;
  trackDelete(request: DeepPartial<TrackDeleteRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  playlistList(
    request: DeepPartial<PlaylistListRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<PlaylistListResponse>;
  playlistGet(request: DeepPartial<PlaylistGetRequest>, options?: CallOptions & CallOptionsExt): Promise<Playlist>;
  playlistCreate(
    request: DeepPartial<PlaylistCreateRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<Playlist>;
  playlistUpdate(
    request: DeepPartial<PlaylistUpdateRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<Playlist>;
  playlistDelete(request: DeepPartial<PlaylistDeleteRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  playlistTrackList(
    request: DeepPartial<PlaylistTrackListRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<PlaylistTrackListResponse>;
  playlistTrackInsert(
    request: DeepPartial<PlaylistTrackInsertRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<Empty>;
  playlistTrackRemove(
    request: DeepPartial<PlaylistTrackRemoveRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<Empty>;
  playlistTrackClear(
    request: DeepPartial<PlaylistTrackClearRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<Empty>;
  import(request: AsyncIterable<DeepPartial<ImportRequest>>, options?: CallOptions & CallOptionsExt): Promise<Track>;
  metadataFetch(request: DeepPartial<MetadataFetchRequest>, options?: CallOptions & CallOptionsExt): Promise<Empty>;
  metadataAlbumTracks(
    request: DeepPartial<MetadataAlbumTracksRequest>,
    options?: CallOptions & CallOptionsExt,
  ): Promise<MetadataAlbumTracksResponse>;
}

function bytesFromBase64(b64: string): Uint8Array {
  if (globalThis.Buffer) {
    return Uint8Array.from(globalThis.Buffer.from(b64, "base64"));
  } else {
    const bin = globalThis.atob(b64);
    const arr = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; ++i) {
      arr[i] = bin.charCodeAt(i);
    }
    return arr;
  }
}

function base64FromBytes(arr: Uint8Array): string {
  if (globalThis.Buffer) {
    return globalThis.Buffer.from(arr).toString("base64");
  } else {
    const bin: string[] = [];
    arr.forEach((byte) => {
      bin.push(globalThis.String.fromCharCode(byte));
    });
    return globalThis.btoa(bin.join(""));
  }
}

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends globalThis.Array<infer U> ? globalThis.Array<DeepPartial<U>>
  : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

function toTimestamp(date: Date): Timestamp {
  const seconds = Math.trunc(date.getTime() / 1_000);
  const nanos = (date.getTime() % 1_000) * 1_000_000;
  return { seconds, nanos };
}

function fromTimestamp(t: Timestamp): Date {
  let millis = (t.seconds || 0) * 1_000;
  millis += (t.nanos || 0) / 1_000_000;
  return new globalThis.Date(millis);
}

function fromJsonTimestamp(o: any): Date {
  if (o instanceof globalThis.Date) {
    return o;
  } else if (typeof o === "string") {
    return new globalThis.Date(o);
  } else {
    return fromTimestamp(Timestamp.fromJSON(o));
  }
}

function isObject(value: any): boolean {
  return typeof value === "object" && value !== null;
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}

export type ServerStreamingMethodResult<Response> = { [Symbol.asyncIterator](): AsyncIterator<Response, void> };
