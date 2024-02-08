use std::borrow::Cow;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    NotFound,
    Invalid,
    Unauthorized,
    Internal,
}

pub struct Error {
    backtrace: std::backtrace::Backtrace,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    kind: ErrorKind,
    message: Cow<'static, str>,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            backtrace: std::backtrace::Backtrace::force_capture(),
            source: None,
            kind,
            message: message.into(),
        }
    }

    pub fn with_source(
        kind: ErrorKind,
        message: impl Into<Cow<'static, str>>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            backtrace: std::backtrace::Backtrace::force_capture(),
            source: Some(Box::new(source)),
            kind,
            message: message.into(),
        }
    }

    pub fn wrap(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            backtrace: std::backtrace::Backtrace::force_capture(),
            source: Some(Box::new(err)),
            kind: ErrorKind::Internal,
            message: "internal error".into(),
        }
    }

    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(f, "{}: {}", self.message, source),
            None => write!(f, "{}", self.message),
        }?;
        f.write_str("\n\n")?;
        for frame in self.backtrace.frames() {
            f.write_str(&format!("{:?}\n", frame))?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(f, "{}: {}", self.message, source),
            None => write!(f, "{}", self.message),
        }?;
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source {
            Some(ref source) => Some(source.as_ref()),
            None => None,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::with_source(ErrorKind::Internal, "database error", error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::with_source(ErrorKind::Internal, "I/O error", error)
    }
}
