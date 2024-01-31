pub trait ResultExt<T> {
    fn m(self) -> Result<T, tonic::Status>;
}

impl<T> ResultExt<T> for sonar::Result<T> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| match e.kind() {
            sonar::ErrorKind::NotFound => tonic::Status::not_found(e.to_string()),
            sonar::ErrorKind::Invalid => tonic::Status::invalid_argument(e.to_string()),
            sonar::ErrorKind::Unauthorized => tonic::Status::unauthenticated(e.to_string()),
            sonar::ErrorKind::Internal => tonic::Status::internal(e.to_string()),
        })
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidIdError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidPropertyKeyError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidPropertyValueError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidGenreError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}

impl<T> ResultExt<T> for sonar::Result<T, sonar::InvalidUsernameError> {
    fn m(self) -> Result<T, tonic::Status> {
        self.map_err(|e| tonic::Status::invalid_argument(e.to_string()))
    }
}
