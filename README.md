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

