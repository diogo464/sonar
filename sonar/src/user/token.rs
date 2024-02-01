use std::{str::FromStr, sync::Arc};

use rand::Rng;

const USER_TOKEN_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const USER_TOKEN_LENGTH: usize = 32;

#[derive(Debug)]
pub struct InvalidUserTokenError;

impl std::fmt::Display for InvalidUserTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid user token")
    }
}

impl std::error::Error for InvalidUserTokenError {}

/// A user token is a random string that is used to authenticate a user session.
/// This is obtained by logging in with a username and password.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserToken(Arc<str>);

impl std::fmt::Display for UserToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserToken {
    type Err = InvalidUserTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::check_token(s)?;
        Ok(Self(Arc::from(s)))
    }
}

impl TryFrom<String> for UserToken {
    type Error = InvalidUserTokenError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::check_token(&value)?;
        Ok(Self(Arc::from(value)))
    }
}

impl TryFrom<&str> for UserToken {
    type Error = InvalidUserTokenError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::check_token(value)?;
        Ok(Self(Arc::from(value)))
    }
}

impl AsRef<str> for UserToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl UserToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        (0..USER_TOKEN_LENGTH)
            .map(|_| rng.gen_range(0..USER_TOKEN_ALPHABET.len()))
            .map(|i| USER_TOKEN_ALPHABET.chars().nth(i).unwrap())
            .collect::<String>()
            .try_into()
            .unwrap()
    }

    fn check_token(token: &str) -> Result<(), InvalidUserTokenError> {
        if token.len() != USER_TOKEN_LENGTH {
            return Err(InvalidUserTokenError);
        }

        if !token.chars().all(|c| USER_TOKEN_ALPHABET.contains(c)) {
            return Err(InvalidUserTokenError);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_user_token() {
        let token = UserToken::random();
        assert_eq!(token.as_str().len(), USER_TOKEN_LENGTH);
        assert!(token
            .as_str()
            .chars()
            .all(|c| USER_TOKEN_ALPHABET.contains(c)));
    }

    #[test]
    fn test_user_token_from_str() {
        let token = UserToken::random();
        let token_str = token.as_str();
        assert_eq!(token_str.parse::<UserToken>().unwrap(), token);
        assert_eq!(UserToken::try_from(token_str).unwrap(), token);
        assert_eq!(UserToken::try_from(token_str.to_owned()).unwrap(), token);
    }

    #[test]
    fn test_user_token_from_str_invalid() {
        assert!(UserToken::try_from("abc").is_err());
    }
}
