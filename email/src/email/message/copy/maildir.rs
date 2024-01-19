use async_trait::async_trait;
use log::info;
use thiserror::Error;

use crate::{envelope::Id, maildir::MaildirSession, Result};

use super::CopyMessages;

#[derive(Error, Debug)]
pub enum Error {
    #[error("maildir: cannot copy messages {3} from folder {1} to folder {2}")]
    CopyMessagesError(#[source] maildirpp::Error, String, String, String),
}

#[derive(Clone)]
pub struct CopyMessagesMaildir {
    session: MaildirSession,
}

impl CopyMessagesMaildir {
    pub fn new(session: &MaildirSession) -> Option<Box<dyn CopyMessages>> {
        let session = session.clone();
        Some(Box::new(Self { session }))
    }
}

#[async_trait]
impl CopyMessages for CopyMessagesMaildir {
    async fn copy_messages(&self, from_folder: &str, to_folder: &str, id: &Id) -> Result<()> {
        info!("maildir: copying messages {id} from folder {from_folder} to folder {to_folder}");

        let session = self.session.lock().await;
        let from_mdir = session.get_maildir_from_folder_name(from_folder)?;
        let to_mdir = session.get_maildir_from_folder_name(to_folder)?;

        id.iter().try_for_each(|id| {
            from_mdir.copy_to(id, &to_mdir).map_err(|err| {
                Error::CopyMessagesError(
                    err,
                    from_folder.to_owned(),
                    to_folder.to_owned(),
                    id.to_owned(),
                )
            })
        })?;

        Ok(())
    }
}
