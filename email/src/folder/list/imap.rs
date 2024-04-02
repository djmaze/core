use async_trait::async_trait;
use log::info;

use crate::{folder::error::Error, imap::ImapContextSync};

use super::{Folders, ListFolders};

#[derive(Debug, Clone)]
pub struct ListImapFolders {
    ctx: ImapContextSync,
}

impl ListImapFolders {
    pub fn new(ctx: &ImapContextSync) -> Self {
        Self { ctx: ctx.clone() }
    }

    pub fn new_boxed(ctx: &ImapContextSync) -> Box<dyn ListFolders> {
        Box::new(Self::new(ctx))
    }

    pub fn some_new_boxed(ctx: &ImapContextSync) -> Option<Box<dyn ListFolders>> {
        Some(Self::new_boxed(ctx))
    }
}

#[async_trait]
impl ListFolders for ListImapFolders {
    async fn list_folders(&self) -> crate::Result<Folders> {
        info!("listing imap folders");

        let config = &self.ctx.account_config;
        let mut ctx = self.ctx.lock().await;

        let names = ctx
            .exec(
                |session| session.list(Some(""), Some("*")),
                |err| Error::ListFoldersImapError(err).into(),
            )
            .await?;

        let folders = Folders::from_imap_names(config, names);

        Ok(folders)
    }
}
