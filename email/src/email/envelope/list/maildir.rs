use async_trait::async_trait;
use log::{debug, info};
use thiserror::Error;

use crate::{
    envelope::Envelope, maildir::MaildirContextSync, search_query::SearchEmailsQuery, Result,
};

use super::{Envelopes, ListEnvelopes, ListEnvelopesOptions};

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot list maildir envelopes from {0}: page {1} out of bounds")]
    GetEnvelopesOutOfBoundsError(String, usize),
}

#[derive(Clone)]
pub struct ListMaildirEnvelopes {
    ctx: MaildirContextSync,
}

impl ListMaildirEnvelopes {
    pub fn new(ctx: &MaildirContextSync) -> Self {
        Self { ctx: ctx.clone() }
    }

    pub fn new_boxed(ctx: &MaildirContextSync) -> Box<dyn ListEnvelopes> {
        Box::new(Self::new(ctx))
    }

    pub fn some_new_boxed(ctx: &MaildirContextSync) -> Option<Box<dyn ListEnvelopes>> {
        Some(Self::new_boxed(ctx))
    }
}

#[async_trait]
impl ListEnvelopes for ListMaildirEnvelopes {
    async fn list_envelopes(&self, folder: &str, opts: ListEnvelopesOptions) -> Result<Envelopes> {
        info!("listing maildir envelopes from folder {folder}");

        let ctx = self.ctx.lock().await;
        let mdir = ctx.get_maildir_from_folder_name(folder)?;

        let mut envelopes = Envelopes::from_mdir_entries(mdir.list_cur(), opts.query.as_ref());
        debug!("maildir envelopes: {envelopes:#?}");

        let page_begin = opts.page * opts.page_size;
        debug!("page begin: {}", page_begin);
        if page_begin > envelopes.len() {
            return Err(
                Error::GetEnvelopesOutOfBoundsError(folder.to_owned(), page_begin + 1).into(),
            );
        }

        let page_end = envelopes.len().min(if opts.page_size == 0 {
            envelopes.len()
        } else {
            page_begin + opts.page_size
        });
        debug!("page end: {}", page_end);

        envelopes.sort_by(|a, b| b.date.partial_cmp(&a.date).unwrap());
        *envelopes = envelopes[page_begin..page_end].into();

        Ok(envelopes)
    }
}

impl SearchEmailsQuery {
    pub fn matches_maildir_search_query(&self, envelope: &Envelope) -> bool {
        match self {
            SearchEmailsQuery::And(left, right) => {
                let left = left.matches_maildir_search_query(envelope);
                let right = right.matches_maildir_search_query(envelope);
                left && right
            }
            SearchEmailsQuery::Or(left, right) => {
                let left = left.matches_maildir_search_query(envelope);
                let right = right.matches_maildir_search_query(envelope);
                left || right
            }
            SearchEmailsQuery::Not(filter) => !filter.matches_maildir_search_query(envelope),
            SearchEmailsQuery::Before(date) => &envelope.date <= date,
            SearchEmailsQuery::After(date) => &envelope.date > date,
            SearchEmailsQuery::From(pattern) => {
                if let Some(name) = &envelope.from.name {
                    if name.contains(pattern) {
                        return true;
                    }
                }
                envelope.from.addr.contains(pattern)
            }
            SearchEmailsQuery::To(pattern) => {
                if let Some(name) = &envelope.to.name {
                    if name.contains(pattern) {
                        return true;
                    }
                }
                envelope.to.addr.contains(pattern)
            }
            SearchEmailsQuery::Subject(pattern) => envelope.subject.contains(pattern),
            SearchEmailsQuery::Body(_pattern) => {
                // TODO
                true
            }
            SearchEmailsQuery::Keyword(pattern) => {
                for flag in envelope.flags.iter() {
                    if flag.to_string().contains(pattern) {
                        return true;
                    }
                }
                false
            }
        }
    }
}
