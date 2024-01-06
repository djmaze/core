#[cfg(feature = "imap")]
pub mod imap;
#[cfg(feature = "maildir")]
pub mod maildir;

use async_trait::async_trait;

use crate::{envelope::Id, Result};

use super::Messages;

#[async_trait]
pub trait PeekMessages: Send + Sync {
    /// Peek email messages from the given folder matching the given
    /// ids.
    ///
    /// When peeking messages, associated envelope flags do not
    /// change. If you want [`Flag::Seen`](crate::email::Flag) to be
    /// automatically added to envelopes, see
    /// [`GetMessages`](super::get::GetMessages).
    async fn peek_messages(&self, folder: &str, id: &Id) -> Result<Messages>;
}
