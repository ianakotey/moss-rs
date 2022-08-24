use std::{net::ToSocketAddrs, path::PathBuf, str::FromStr};

use getset_scoped::{Getters, Setters};

use crate::prelude::MossLanguage;

#[derive(Debug, Default, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
#[allow(dead_code)] // TODO: Remove
pub struct MossConfig<S: ToSocketAddrs> {
    #[getset(get = "pub")]
    server_address: S,

    user_id: String,

    comment: String,
    language: MossLanguage,
    use_directory_mode: bool,

    use_experimental_mode: bool,
    max_matches_displayed: usize,
    max_ignore_threshold: usize,

    #[getset(skip)]
    _base_files: Vec<PathBuf>,
    #[getset(skip)]
    _base_globs: Vec<String>,

    #[getset(skip)]
    _submission_files: Vec<PathBuf>,
    #[getset(skip)]
    _submission_globs: Vec<String>,
}


impl<S: ToSocketAddrs> MossConfig<S> {
    pub fn new<U: ToString>(user_id: U, server_address: S) -> Self {
        MossConfig {
            server_address,
            user_id: user_id.to_string(),
            use_experimental_mode: Default::default(),
            max_matches_displayed: 250,
            max_ignore_threshold: 10,
            _base_files: Default::default(),
            _base_globs: Default::default(),
            _submission_files: Default::default(),
            _submission_globs: Default::default(),
            comment: Default::default(),
            language: Default::default(),
            use_directory_mode: Default::default(),
        }

    }

    pub fn add_base_file<P: AsRef<str> + ToString>(&mut self, path: &P) -> &mut Self {
        let p = PathBuf::from_str(path.as_ref()).unwrap();
        if p.exists() {
            self._base_files.push(p);
        }
        self
    }

    pub fn add_file<P: AsRef<str> + ToString>(&mut self, path: &P) -> &mut Self {
        let p = PathBuf::from_str(path.as_ref()).unwrap();
        if p.exists() {
            self._submission_files.push(p);
        }
        self
    }

    pub fn add_base_file_by_glob<P: ToString>(&mut self, glob: &P) -> &mut Self {
        self._base_globs.push(glob.to_string());
        self
    }

    pub fn add_file_by_glob<P: ToString>(&mut self, glob: &P) -> &mut Self {
        self._submission_globs.push(glob.to_string());
        self
    }

    pub fn base_files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        // self._base_globs
        //     .iter()
        //     .map(|glob| shellexpand::full(glob))
        //     .inspect(|x| ()) // log invalid globs here
        //     .flatten() // remove previously logged invalid globs
        //     .map(|pattern| glob::glob(pattern.as_ref()))
        //     .inspect(|x| ()) // log invalid patterns here
        //     .flatten() // remove previously logged invalid patterns
        //     .flatten() // merge the iterators for each glob into one iterator
        //     .inspect(|x| ()) // log inaccessible paths here
        //     .flatten() // remove previously logged inaccessible paths
        //     // return an iterator over the files by copying from the original vector on-demand
        //     // chain this iterator to the globs created above
        //     .chain(self._base_files.iter().cloned())

        std::iter::empty()
    }

    pub fn submission_files<'a>(&'a self) -> impl Iterator<Item = PathBuf> + 'a {
        self._submission_globs
            .iter()
            .map(|glob| shellexpand::full(glob))
            .inspect(|x| ()) // log invalid globs here
            .flatten() // remove previously logged invalid globs
            .map(|pattern| glob::glob(pattern.as_ref()))
            .inspect(|x| ()) // log invalid patterns here
            .flatten() // remove previously logged invalid patterns
            .flatten() // merge the iterators for each glob into one big iterator
            .inspect(|x| ()) // log inaccessible paths here
            .flatten() // remove previously logged inaccessible paths
            // return an iterator over the files by copying from the original vector on-demand
            // chain this iterator to the globs created above
            .chain(self._submission_files.iter().cloned())
    }
}
