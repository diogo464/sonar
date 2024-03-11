use std::time::Duration;

use bytes::Bytes;

use crate::{
    bytestream::ByteStream,
    extractor::{ExtractedMetadata, Extractor},
    Album, AlbumId, Artist, ArtistId, Audio, AudioId, Context, Playlist, Track, User, UserId,
};

// https://gist.github.com/scotthaleen/32f76a413e0dfd4b4d79c2a534d49c0b
pub const SMALL_IMAGE_JPEG: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00, 0x48,
    0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC2, 0x00, 0x0B, 0x08, 0x00, 0x01,
    0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08,
    0x01, 0x01, 0x00, 0x01, 0x3F, 0x10,
];

// ffmpeg -f lavfi -i anullsrc=r=44100:cl=mono -t 1 -q:a 9 -acodec libmp3lame silence.mp3
pub const SMALL_AUDIO_MP3_DURATION: Duration = Duration::from_millis(1040);
pub const SMALL_AUDIO_MP3: &[u8] = &[
    0x49, 0x44, 0x33, 0x4, 0x0, 0x0, 0x0, 0x0, 0x0, 0x23, 0x54, 0x53, 0x53, 0x45, 0x0, 0x0, 0x0,
    0xF, 0x0, 0x0, 0x3, 0x4C, 0x61, 0x76, 0x66, 0x36, 0x30, 0x2E, 0x31, 0x36, 0x2E, 0x31, 0x30,
    0x30, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xFF, 0xFB, 0x40, 0xC0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x49, 0x6E, 0x66,
    0x6F, 0x0, 0x0, 0x0, 0xF, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x10, 0xF6, 0x0, 0x10, 0x10, 0x16,
    0x16, 0x1D, 0x1D, 0x1D, 0x23, 0x23, 0x29, 0x29, 0x29, 0x2F, 0x2F, 0x35, 0x35, 0x35, 0x3B, 0x3B,
    0x41, 0x41, 0x41, 0x48, 0x48, 0x4E, 0x4E, 0x4E, 0x54, 0x54, 0x5A, 0x5A, 0x5A, 0x60, 0x60, 0x66,
    0x66, 0x66, 0x6C, 0x6C, 0x72, 0x72, 0x72, 0x79, 0x79, 0x7F, 0x7F, 0x7F, 0x85, 0x85, 0x8B, 0x8B,
    0x8B, 0x91, 0x91, 0x97, 0x97, 0x97, 0x9D, 0x9D, 0xA4, 0xA4, 0xA4, 0xAA, 0xAA, 0xB0, 0xB0, 0xB0,
    0xB6, 0xB6, 0xBC, 0xBC, 0xBC, 0xC2, 0xC2, 0xC8, 0xC8, 0xC8, 0xCE, 0xCE, 0xD5, 0xD5, 0xD5, 0xDB,
    0xDB, 0xE1, 0xE1, 0xE1, 0xE7, 0xE7, 0xED, 0xED, 0xED, 0xF3, 0xF3, 0xF9, 0xF9, 0xF9, 0xFF, 0xFF,
    0x0, 0x0, 0x0, 0x0, 0x4C, 0x61, 0x76, 0x63, 0x36, 0x30, 0x2E, 0x33, 0x31, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x24, 0x5, 0x7C, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10,
    0xF6, 0x29, 0xF8, 0xD3, 0xA9, 0x0, 0x0, 0x0, 0x0, 0x0, 0xFF, 0xFB, 0x10, 0xC4, 0x0, 0x3, 0xC0,
    0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x4C, 0x41, 0x4D,
    0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55,
    0xFF, 0xFB, 0x10, 0xC4, 0x29, 0x83, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34,
    0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E,
    0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0x53, 0x3, 0xC0, 0x0, 0x1,
    0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB,
    0x10, 0xC4, 0x7C, 0x83, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0,
    0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30,
    0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xA6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0,
    0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C,
    0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4,
    0xCF, 0x83, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55,
    0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20,
    0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D,
    0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3,
    0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55,
    0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0,
    0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33,
    0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0,
    0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF,
    0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80,
    0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31,
    0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4,
    0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10,
    0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0,
    0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30,
    0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0,
    0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41,
    0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6,
    0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55,
    0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0,
    0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45,
    0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0,
    0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55,
    0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34,
    0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E,
    0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1,
    0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB,
    0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0,
    0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30,
    0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0,
    0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C,
    0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4,
    0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55,
    0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20,
    0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D,
    0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3,
    0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55,
    0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0,
    0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33,
    0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0,
    0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF,
    0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80,
    0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31,
    0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4,
    0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10,
    0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0,
    0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30,
    0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0,
    0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41,
    0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6,
    0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55,
    0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0,
    0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45,
    0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0,
    0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E, 0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55,
    0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34,
    0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x4C, 0x41, 0x4D, 0x45, 0x33, 0x2E,
    0x31, 0x30, 0x30, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1,
    0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB,
    0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0,
    0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0,
    0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4,
    0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3, 0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20,
    0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0xFF, 0xFB, 0x10, 0xC4, 0xD6, 0x3,
    0xC0, 0x0, 0x1, 0xA4, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x34, 0x80, 0x0, 0x0, 0x4, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
    0x55,
];

#[derive(Debug, Clone)]
pub struct StaticMetadataExtractor(ExtractedMetadata);

impl StaticMetadataExtractor {
    pub fn new(metadata: ExtractedMetadata) -> Self {
        Self(metadata)
    }
}

impl Extractor for StaticMetadataExtractor {
    fn extract(&self, _path: &std::path::Path) -> std::io::Result<ExtractedMetadata> {
        Ok(self.0.clone())
    }
}

pub fn create_config_memory() -> crate::Config {
    crate::Config::new(
        ":memory:",
        crate::StorageBackend::Memory,
        crate::SearchBackend::BuiltIn,
    )
}

pub async fn create_context_memory() -> Context {
    let config = create_config_memory();
    create_context(config).await
}

pub async fn create_context(config: crate::Config) -> Context {
    crate::new(config).await.unwrap()
}

pub async fn create_user(ctx: &Context, username: &str) -> User {
    create_user_with_password(ctx, username, "password").await
}

pub async fn create_user_with_password(ctx: &Context, username: &str, password: &str) -> User {
    crate::user_create(
        ctx,
        crate::UserCreate {
            username: username.parse().unwrap(),
            password: password.to_owned(),
            avatar: None,
            admin: true,
        },
    )
    .await
    .unwrap()
}

pub async fn create_artist(ctx: &Context, name: &str) -> Artist {
    crate::artist_create(
        ctx,
        crate::ArtistCreate {
            name: name.to_string(),
            cover_art: None,
            genres: Default::default(),
            properties: Default::default(),
        },
    )
    .await
    .unwrap()
}

pub async fn create_album(ctx: &Context, artist: ArtistId, name: &str) -> Album {
    crate::album_create(
        ctx,
        crate::AlbumCreate {
            artist,
            name: name.to_string(),
            cover_art: None,
            genres: Default::default(),
            properties: Default::default(),
        },
    )
    .await
    .unwrap()
}

pub async fn create_track(ctx: &Context, album: AlbumId, name: &str) -> Track {
    create_track_with_audio_opt(ctx, album, name, None).await
}

pub async fn create_artist_album_track(
    ctx: &Context,
    artist_name: &str,
    album_name: &str,
    track_name: &str,
) -> (Artist, Album, Track) {
    let artist = create_artist(ctx, artist_name).await;
    let album = create_album(ctx, artist.id, album_name).await;
    let track = create_track(ctx, album.id, track_name).await;
    (artist, album, track)
}

pub async fn create_track_with_audio(
    ctx: &Context,
    album: AlbumId,
    name: &str,
    audio: AudioId,
) -> Track {
    create_track_with_audio_opt(ctx, album, name, Some(audio)).await
}

pub async fn create_track_with_audio_opt(
    ctx: &Context,
    album: AlbumId,
    name: &str,
    audio: Option<AudioId>,
) -> Track {
    crate::track_create(
        ctx,
        crate::TrackCreate {
            name: name.to_string(),
            album,
            cover_art: None,
            lyrics: None,
            audio,
            properties: Default::default(),
        },
    )
    .await
    .unwrap()
}

pub async fn create_audio(ctx: &Context, data: &[u8]) -> Audio {
    crate::audio_create(
        ctx,
        crate::AudioCreate {
            stream: create_stream(data),
            filename: Some("test.mp3".to_string()),
        },
    )
    .await
    .unwrap()
}

pub async fn create_playlist(ctx: &Context, owner: UserId, name: &str) -> Playlist {
    crate::playlist_create(
        ctx,
        crate::PlaylistCreate {
            name: name.to_string(),
            owner,
            tracks: Default::default(),
            properties: Default::default(),
        },
    )
    .await
    .unwrap()
}

pub fn create_simple_genres() -> crate::Genres {
    let mut genres = crate::Genres::default();
    genres.set(&"heavy metal".parse().unwrap());
    genres.set(&"electronic".parse().unwrap());
    genres
}

pub fn create_simple_properties() -> crate::Properties {
    let mut properties = crate::Properties::default();
    properties.insert(
        crate::PropertyKey::new_uncheked("key1"),
        crate::PropertyValue::new_uncheked("value1"),
    );
    properties.insert(
        crate::PropertyKey::new_uncheked("key2"),
        crate::PropertyValue::new_uncheked("value2"),
    );
    properties
}

pub fn create_stream(data: &[u8]) -> ByteStream {
    crate::bytestream::from_bytes(Bytes::from(data.to_vec()))
}
