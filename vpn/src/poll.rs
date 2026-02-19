#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Token<ID = i32> {
    Tun,
    Sock(ID),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UnknownToken;

impl<ID> From<Token<ID>> for u64
where
    ID: Into<i32>,
{
    fn from(token: Token<ID>) -> Self {
        match token {
            Token::Tun => 1 << 32,
            Token::Sock(id) => 2 << 32 | (id.into() as u32 as u64),
        }
    }
}

impl<ID> TryFrom<u64> for Token<ID>
where
    ID: From<i32>,
{
    type Error = UnknownToken;

    fn try_from(id: u64) -> Result<Self, Self::Error> {
        let tag = id >> 32;
        let token = match tag {
            1 => Token::Tun,
            2 => Token::Sock((id as i32).into()),
            _ => return Err(UnknownToken),
        };

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_into_roundtrip() {
        for token in [
            Token::Tun,
            Token::Sock(i32::MIN),
            Token::Sock(-1),
            Token::Sock(0),
            Token::Sock(4),
            Token::Sock(i32::MAX),
        ] {
            let num: u64 = token.into();
            assert_eq!(num.try_into(), Ok(token));
        }
    }
}
