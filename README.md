# sonar

Self hosted music database/streaming server.

## Server Environment Variables

```
# listen address for grpc api
SONAR_ADDRESS="0.0.0.0:3000"
# listen address for opensubsonic api
SONAR_OPENSUBSONIC_ADDRESS="0.0.0.0:3001"
# data directory
SONAR_DATA_DIR="./"
# default admin username. created if it does not already exist.
SONAR_DEFAULT_ADMIN_USERNAME
# default admin password.
SONAR_DEFAULT_ADMIN_PASSWORD

## spotify integration (optional)
# spotify username for the account used to download songs.
SONAR_SPOTIFY_USERNAME="..."
# spotify password
SONAR_SPOTIFY_PASSWORD="..."
# spotify api client id
SONAR_SPOTIFY_CLIENT_ID="..."
# spotify api secret key
SONAR_SPOTIFY_CLIENT_SECRET="..."
```
# opensubsonic

TODO:
+ some authentication middleware
+ rename get_starreed to get_starred
+ maybe add error middleware

## opensubsonic
This crate provides types to work with the [OpenSubsonic API](https://opensubsonic.netlify.app/).
Right now it is mainly focused on implementing server-side functionality.

### Server

A server can be created by implementing the [`service::OpenSubsonicServer`] trait.
Then a [`tower::Service`] can be created from it using [`service::OpenSubsonicService::new`].
An example can be found in the [`service`] module.

## opensubsonic-cli

```
Usage: opensubsonic [OPTIONS] <COMMAND>

Commands:
  add-chat-message               Adds a message to the chat log
  change-password                Changes the password of an existing Subsonic user, using the following parameters. You can only change your own password unless you have admin privileges
  create-bookmark                Creates or updates a bookmark (a position within a media file). Bookmarks are personal and not visible to other users
  create-internet-radio-station  Adds a new internet radio station. Only users with admin privileges are allowed to call this method
  create-playlist                Creates (or updates) a playlist
  create-podcast-channel         Adds a new Podcast channel. Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts)
  create-share                   Creates a public URL that can be used by anyone to stream music or video from the Subsonic server. The URL is short and suitable for posting on Facebook, Twitter etc. Note: The user must be authorized to share (see Settings > Users > User is allowed to share files with anyone)
  create-user                    Creates a new Subsonic user, using the following parameters
  delete-bookmark                Deletes the bookmark for a given file
  delete-internet-radio-station  Deletes an existing internet radio station. Only users with admin privileges are allowed to call this method
  delete-playlist                Deletes a saved playlist
  delete-podcast-channel         Deletes a Podcast channel. Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts)
  delete-podcast-episode         Deletes a Podcast episode. Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts)
  delete-share                   Deletes an existing share
  delete-user                    Deletes an existing Subsonic user, using the following parameters
  download                       Downloads a given media file. Similar to [`Stream`], but this method returns the original media data without transcoding or downsampling
  download-podcast-episode       Request the server to start downloading a given Podcast episode. Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts)
  get-album                      Returns details for an album, including a list of songs
  get-album-info                 Returns album notes, image URLs etc, using data from <last.fm>
  get-album-info2                Similar to [`GetAlbumInfo`], but organizes music according to ID3 tags
  get-album-list                 Returns a list of random, newest, highest rated etc. albums. Similar to the album lists on the home page of the Subsonic web interface
  get-album-list2                Similar to [`GetAlbumList`], but organizes music according to ID3 tags
  get-artist                     Returns details for an artist, including a list of albums
  get-artist-info                Returns artist info with biography, image URLs and similar artists, using data from <http://last.fm>
  get-artist-info2               Similar to [`GetArtistInfo`], but organizes music according to ID3 tags
  get-artists                    Represents the parameters for the `getArtists` request
  get-avatar                     Returns the avatar (personal image) for a user
  get-bookmarks                  Returns all bookmarks for this user. A bookmark is a position within a certain media file
  get-captions                   Returns captions (subtitles) for a video. Use getVideoInfo to get a list of available captions
  get-chat-messages              Returns the current visible (non-expired) chat messages
  get-cover-art                  Returns a cover art image
  get-genres                     Returns all genres
  get-indexes                    Returns an indexed structure of all artists
  get-internet-radio-stations    Returns all internet radio stations. Takes no extra parameters
  get-license                    <http://www.subsonic.org/pages/api.jsp#getLicense>
  get-lyrics                     Searches for and returns lyrics for a given song
  get-music-directory            Returns a listing of all files in a music directory. Typically used to get list of albums for an artist, or list of songs for an album
  get-music-folders              Returns all configured top-level music folders. Takes no extra parameters
  get-newest-podcasts            Returns the most recently published Podcast episodes
  get-now-playing                Returns what is currently being played by all users. Takes no extra parameters
  get-playlist                   Returns a listing of files in a saved playlist
  get-playlists                  Returns all playlists a user is allowed to play
  get-play-queue                 Returns the state of the play queue for this user (as set by [`SavePlayQueue`]). This includes the tracks in the play queue, the currently playing track, and the position within this track. Typically used to allow a user to move between different clients/apps while retaining the same play queue (for instance when listening to an audio book)
  get-podcasts                   Returns all Podcast channels the server subscribes to, and (optionally) their episodes. This method can also be used to return details for only one channel - refer to the id parameter. A typical use case for this method would be to first retrieve all channels without episodes, and then retrieve all episodes for the single channel the user selects
  get-random-songs               Returns random songs matching the given criteria
  get-scan-status                Returns the current status for media library scanning. Takes no extra parameters
  get-shares                     Returns information about shared media this user is allowed to manage. Takes no extra parameters
  get-similar-songs              Returns a random collection of songs from the given artist and similar artists, using data from last.fm. Typically used for artist radio features
  get-similar-songs2             Similar to [`GetSimilarSongs`], but organizes music according to ID3 tags
  get-song                       Returns details for a song
  get-songs-by-genre             Returns songs in a given genre
  get-starred                    Returns starred songs, albums and artists
  get-starred2                   Returns starred songs, albums and artists
  get-top-songs                  Returns top songs for the given artist, using data from <last.fm>
  get-user                       Get details about a given user, including which authorization roles and folder access it has. Can be used to enable/disable certain features in the client, such as jukebox control
  get-users                      Get details about all users, including which authorization roles and folder access they have. Only users with admin privileges are allowed to call this method
  get-video-info                 Returns details for a video, including information about available audio tracks, subtitles (captions) and conversions
  get-videos                     Represents a request to retrieve all video files
  hls                            Creates an HLS (HTTP Live Streaming) playlist used for streaming video or audio. HLS is a streaming protocol implemented by Apple and works by breaking the overall stream into a sequence of small HTTP-based file downloads. It's supported by iOS and newer versions of Android. This method also supports adaptive bitrate streaming, see the bitRate parameter
  jubebox-control                Controls the jukebox, i.e., playback directly on the server's audio hardware. Note: The user must be authorized to control the jukebox (see Settings > Users > User is allowed to play files in jukebox mode)
  ping                           <http://www.subsonic.org/pages/api.jsp#ping>
  refresh-podcasts               Requests the server to check for new Podcast episodes. Note: The user must be authorized for Podcast administration (see Settings > Users > User is allowed to administrate Podcasts)
  save-play-queue                Saves the state of the play queue for this user. This includes the tracks in the play queue, the currently playing track, and the position within this track. Typically used to allow a user to move between different clients/apps while retaining the same play queue (for instance when listening to an audio book)
  scrobble                       Registers the local playback of one or more media files. Typically used when playing media that is cached on the client. This operation includes the following: - "Scrobbles" the media files on last.fm if the user has configured his/her last.fm credentials on the Subsonic server (Settings > Personal). - Updates the play count and last played timestamp for the media files. (Since 1.11.0) - Makes the media files appear in the "Now playing" page in the web app, and appear in the list of songs returned by [`GetNowPlaying`] (Since 1.11.0) Since 1.8.0 you may specify multiple id (and optionally time) parameters to scrobble multiple files
  search                         Returns a listing of files matching the given search criteria. Supports paging through the result
  search2                        Returns albums, artists and songs matching the given search criteria. Supports paging through the result
  search3                        Similar to [`Search2`], but organizes music according to ID3 tags
  set-rating                     Sets the rating for a music file
  star                           Attaches a star to a song, album or artist
  start-scan                     Initiates a rescan of the media libraries. Takes no extra parameters
  unstar                         Removes the star from a song, album or artist
  update-internet-radio-station  Updates an existing internet radio station. Only users with admin privileges are allowed to call this method
  update-playlist                Updates a playlist. Only the owner of a playlist is allowed to update it
  update-share                   Updates the description and/or expiration date for an existing share
  update-user                    Modifies an existing Subsonic user, using the following parameters
  help                           Print this message or the help of the given subcommand(s)

Options:
      --server <SERVER>      The server to connect to [env: OPENSUBSONIC_SERVER=] [default: http://localhost:3000]
      --client <CLIENT>      The client string to use [env: OPENSUBSONIC_CLIENT=] [default: opensubsonic-cli]
      --username <USERNAME>  The username to use [env: OPENSUBSONIC_USERNAME=]
      --password <PASSWORD>  The password to use [env: OPENSUBSONIC_PASSWORD=]
      --format <FORMAT>      The format to use [env: OPENSUBSONIC_FORMAT=] [default: json]
  -h, --help                 Print help
```
