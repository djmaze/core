use crate::info;
use async_trait::async_trait;

use crate::{email::error::Error, envelope::Id, maildir::MaildirContextSync, AnyResult};

use super::MoveMessages;

#[derive(Clone)]
pub struct MoveMaildirMessages {
    pub(crate) ctx: MaildirContextSync,
}

impl MoveMaildirMessages {
    pub fn new(ctx: &MaildirContextSync) -> Self {
        Self { ctx: ctx.clone() }
    }

    pub fn new_boxed(ctx: &MaildirContextSync) -> Box<dyn MoveMessages> {
        Box::new(Self::new(ctx))
    }

    pub fn some_new_boxed(ctx: &MaildirContextSync) -> Option<Box<dyn MoveMessages>> {
        Some(Self::new_boxed(ctx))
    }
}

#[async_trait]
impl MoveMessages for MoveMaildirMessages {
    async fn move_messages(&self, from_folder: &str, to_folder: &str, id: &Id) -> AnyResult<()> {
        info!("moving maildir messages {id} from folder {from_folder} to folder {to_folder}");

        let ctx = self.ctx.lock().await;
        let from_mdir = ctx.get_maildir_from_folder_name(from_folder)?;
        let to_mdir = ctx.get_maildir_from_folder_name(to_folder)?;

        id.iter().try_for_each(|id| {
            from_mdir.move_to(id, &to_mdir).map_err(|err| {
                Error::MoveMessagesMaildirError(
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
