//! Incremental in-process use for the IDE.
//!
//! A session caches the parsed style (and the transcript prefix it wrote),
//! and memoizes the last run by a hash of the `.idx` inputs.  When LaTeX
//! re-runs and the `.idx` bytes are unchanged, no work is done; when they
//! change, only scanning, sorting and generation run.  `ind_changed` tells
//! the compiler whether another LaTeX pass is needed for the index.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{Fatal, Job, NamedBytes, Output, Prepared, prepare, run_prepared};

pub struct IndexSession {
    template: Job,
    prepared_key: Option<u64>,
    prepared: Option<(crate::Style, Vec<u8>, bool)>,
    last_inputs: Option<u64>,
    last: Option<Output>,
}

#[derive(Debug)]
pub enum SessionUpdate<'a> {
    /// Inputs identical to the previous update; cached output returned.
    Unchanged(&'a Output),
    /// Re-generated; `ind_changed` compares with the previous `.ind` bytes.
    Regenerated { output: &'a Output, ind_changed: bool },
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

impl IndexSession {
    /// `template` supplies style, options and output names; its `inputs`
    /// are ignored in favour of those passed to [`IndexSession::update`].
    pub fn new(template: Job) -> Self {
        IndexSession { template, prepared_key: None, prepared: None, last_inputs: None, last: None }
    }

    /// Replace style/options; invalidates caches.
    pub fn set_job(&mut self, template: Job) {
        *self = IndexSession::new(template);
    }

    pub fn update(&mut self, inputs: Vec<NamedBytes>) -> Result<SessionUpdate<'_>, Fatal> {
        let key = hash(&inputs);
        if self.last_inputs == Some(key) && self.last.is_some() {
            return Ok(SessionUpdate::Unchanged(self.last.as_ref().unwrap()));
        }
        let mut job = self.template.clone();
        job.inputs = inputs;
        // The style scan only depends on the style bytes, options and log.
        let pkey = hash(&(&job.style, &job.log, &job.options, job.inputs.first().map(|i| &i.name)));
        if self.prepared_key != Some(pkey) {
            let (p, page) = prepare(&job)?;
            let has_page = page.is_some();
            self.prepared = Some((p.style, p.log.buf, has_page));
            self.prepared_key = Some(pkey);
        }
        // Rebuild page start (cheap) through prepare's logic when needed.
        let page = if self.prepared.as_ref().unwrap().2 { prepare(&job)?.1 } else { None };
        let (style, prefix, _) = self.prepared.clone().unwrap();
        let mut log = crate::io::Transcript::new();
        log.buf = prefix;
        let out = run_prepared(&job, Prepared { style, log }, page);
        let ind_changed = self.last.as_ref().map(|l| l.ind != out.ind).unwrap_or(true);
        self.last = Some(out);
        self.last_inputs = Some(key);
        Ok(SessionUpdate::Regenerated { output: self.last.as_ref().unwrap(), ind_changed })
    }
}
