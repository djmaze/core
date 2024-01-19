use async_trait::async_trait;
use log::info;
use thiserror::Error;

use crate::{envelope::Id, maildir::MaildirSessionSync, Result};

use super::{AddFlags, Flags};

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot add flags {3} to envelope(s) {2} from folder {1}")]
    AddFlagsError(#[source] maildirpp::Error, String, String, Flags),
}

#[derive(Clone)]
pub struct AddFlagsMaildir {
    session: MaildirSessionSync,
}

impl AddFlagsMaildir {
    pub fn new(session: MaildirSessionSync) -> Self {
        Self { session }
    }

    pub fn new_boxed(session: MaildirSessionSync) -> Box<dyn AddFlags> {
        Box::new(Self::new(session))
    }
}

#[async_trait]
impl AddFlags for AddFlagsMaildir {
    async fn add_flags(&self, folder: &str, id: &Id, flags: &Flags) -> Result<()> {
        info!("maildir: adding flag(s) {flags} to envelope {id} from folder {folder}");

        let session = self.session.lock().await;
        let mdir = session.get_maildir_from_folder_name(folder)?;

        id.iter().try_for_each(|ref id| {
            mdir.add_flags(id, &flags.to_mdir_string()).map_err(|err| {
                Error::AddFlagsError(err, folder.to_owned(), id.to_string(), flags.clone())
            })
        })?;

        Ok(())
    }
}
