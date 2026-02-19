#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Token<ID = i32> {
    Tun,
    Socket(ID),
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
            Token::Socket(id) => 2 << 32 | (id.into() as u32 as u64),
        }
    }
}
