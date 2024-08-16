use async_trait::async_trait;
use imap_next::imap_types::sequence::{Sequence, SequenceSet};
use utf7_imap::encode_utf7_imap as encode_utf7;

use super::{AddFlags, Flags};
use crate::{debug, envelope::Id, imap::ImapContextSync, info, AnyResult, Error};

#[derive(Clone, Debug)]
pub struct AddImapFlags {
    ctx: ImapContextSync,
}

impl AddImapFlags {
    pub fn new(ctx: &ImapContextSync) -> Self {
        Self { ctx: ctx.clone() }
    }

    pub fn new_boxed(ctx: &ImapContextSync) -> Box<dyn AddFlags> {
        Box::new(Self::new(ctx))
    }

    pub fn some_new_boxed(ctx: &ImapContextSync) -> Option<Box<dyn AddFlags>> {
        Some(Self::new_boxed(ctx))
    }
}

#[async_trait]
impl AddFlags for AddImapFlags {
    async fn add_flags(&self, folder: &str, id: &Id, flags: &Flags) -> AnyResult<()> {
        info!("adding imap flag(s) {flags} to envelope {id} from folder {folder}");

        let mut ctx = self.ctx.lock().await;
        let config = &ctx.account_config;

        let folder = config.get_folder_alias(folder);
        let folder_encoded = encode_utf7(folder.clone());
        debug!("utf7 encoded folder: {folder_encoded}");

        let uids: SequenceSet = match id {
            Id::Single(id) => Sequence::try_from(id.as_str())
                .map_err(Error::ParseSequenceError)?
                .into(),
            Id::Multiple(ids) => ids
                .iter()
                .filter_map(|id| {
                    let seq = Sequence::try_from(id.as_str());

                    #[cfg(feature = "tracing")]
                    if let Err(err) = &seq {
                        tracing::debug!(?id, ?err, "skipping invalid sequence");
                    }

                    seq.ok()
                })
                .collect::<Vec<_>>()
                .try_into()
                .map_err(Error::ParseSequenceError)?,
        };

        ctx.select_mailbox(&folder_encoded).await?;
        ctx.add_flags(uids, flags.to_imap_flags_iter()).await?;

        Ok(())
    }
}
