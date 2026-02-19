use std::os::fd::AsFd;

use nix::{
    poll::PollTimeout,
    sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags},
};

use crate::{Error, Result};

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

    fn try_from(id: u64) -> core::result::Result<Self, Self::Error> {
        let tag = id >> 32;
        let token = match tag {
            1 => Token::Tun,
            2 => Token::Sock((id as i32).into()),
            _ => return Err(UnknownToken),
        };

        Ok(token)
    }
}

/// Configure `epoll` behavior
/// - `EpollFlags::EPOLLIN`: Indicates that the file descriptor is ready for reading.
/// - `EpollFlags::EPOLLET`: Enables edge-triggered behavior, meaning notifications
///   are sent only when the state of the file descriptor changes.
const EPOLL_FLAGS: EpollFlags = EpollFlags::EPOLLIN.union(EpollFlags::EPOLLET);

pub struct Poll {
    epoll: Epoll,
}

impl Poll {
    pub fn new() -> Result<Self> {
        // close epoll fd in case a child is spawned
        let epoll = Epoll::new(EpollCreateFlags::EPOLL_CLOEXEC)?;
        Ok(Poll { epoll })
    }

    pub fn register_read<F: AsFd, ID: Into<i32>>(&self, fd: F, token: Token<ID>) -> Result<()> {
        let event = EpollEvent::new(EPOLL_FLAGS, token.into());
        self.epoll.add(fd, event)?;

        Ok(())
    }

    pub fn delete<F: AsFd>(&self, fd: F) -> Result<()> {
        self.epoll.delete(fd)?;
        Ok(())
    }

    pub fn wait<ID: From<i32>>(&self) -> Result<Token<ID>> {
        let mut events = [EpollEvent::empty()];

        let n = self.epoll.wait(&mut events, PollTimeout::NONE)?;
        assert_eq!(n, 1);

        let data = events[0].data();
        let token = Token::try_from(data).map_err(|_| Error::Token(data))?;

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
