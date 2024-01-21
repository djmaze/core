use async_trait::async_trait;
use log::{info, trace};
use std::collections::HashMap;

use crate::{
    folder::{Folder, Folders},
    notmuch::NotmuchContextSync,
    Result,
};

use super::ListFolders;

pub struct ListNotmuchFolders {
    ctx: NotmuchContextSync,
}

impl ListNotmuchFolders {
    pub fn new(ctx: &NotmuchContextSync) -> Self {
        Self { ctx: ctx.clone() }
    }

    pub fn new_boxed(ctx: &NotmuchContextSync) -> Box<dyn ListFolders> {
        Box::new(Self::new(ctx))
    }

    pub fn some_new_boxed(ctx: &NotmuchContextSync) -> Option<Box<dyn ListFolders>> {
        Some(Self::new_boxed(ctx))
    }
}

#[async_trait]
impl ListFolders for ListNotmuchFolders {
    async fn list_folders(&self) -> Result<Folders> {
        info!("listing notmuch virtual folders");

        let mut folders: Folders = self
            .ctx
            .account_config
            .get_folder_aliases()
            .unwrap_or(&HashMap::default())
            .into_iter()
            .map(|(name, alias)| Folder {
                kind: None,
                name: name.into(),
                desc: alias.into(),
            })
            .collect();

        folders.sort_by(|a, b| b.name.partial_cmp(&a.name).unwrap());

        trace!("notmuch virtual folders: {folders:#?}");

        Ok(folders)
    }
}
