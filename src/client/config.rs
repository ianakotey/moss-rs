use std::{net::ToSocketAddrs, path::PathBuf, str::FromStr};

use getset_scoped::{Getters, Setters};
use snafu::{prelude::*, Whatever};

use crate::prelude::MossLanguage;

#[derive(Debug, Default, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
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

    transform: Option<String>,

    #[getset(skip)]
    _base_files: Vec<PathBuf>,

    #[getset(skip)]
    _submission_files: Vec<PathBuf>,
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
            _submission_files: Default::default(),
            comment: Default::default(),
            language: Default::default(),
            use_directory_mode: Default::default(),
            transform: None,
        }
    }

    pub fn add_base_file<P: AsRef<str>>(&mut self, path: &P) -> Result<(), Whatever> {
        Self::_add_file_to_vec(path, &mut self._base_files)
    }

    pub fn add_file<P: AsRef<str>>(&mut self, path: &P) -> Result<(), Whatever> {
        Self::_add_file_to_vec(path, &mut self._submission_files)
    }
    fn _add_file_to_vec<P: AsRef<str>>(path: &P, vec: &mut Vec<PathBuf>) -> Result<(), Whatever> {
        let p = match PathBuf::from_str(path.as_ref()) {
            Ok(it) => it,
            _ => unreachable!(), // <PathBuf as FromStr>::Err = Infallible
        };

        if p.exists() {
            vec.push(p);
            Ok(())
        } else {
            // Assume the path is a glob, and try to expand it
            let full_path = shellexpand::full(path).with_whatever_context(|err| {
                format!("Unable to expand variable in path: {}", err.var_name)
            })?;
            let matches = glob::glob(&full_path)
                .with_whatever_context(|err| format!("Invalid glob pattern passed: {}", err.msg))?;

            let mut unreadable_paths = Vec::new();

            matches
                .filter_map(|x| match x {
                    // filter out unreachable paths,
                    // but keep them for error reporting
                    Ok(path) => Some(path),
                    Err(err) => {
                        unreadable_paths.push(err);
                        None
                    }
                })
                .for_each(|p| vec.push(p));

            // return custom type with all unreadable paths
            unreadable_paths
                .is_empty()
                .then_some(())
                .whatever_context("Some paths were invalid")
        }
    }

    fn _add_path_to_vec(path: PathBuf, vec: &mut Vec<PathBuf>) -> Result<(), Whatever> {
        path.exists()
            .then(|| {
                vec.push(path.clone());
            })
            .with_whatever_context(|| format!("Path does not exist: {:?}", path))
    }
    pub fn add_path(&mut self, path: PathBuf) -> Result<(), Whatever> {
        Self::_add_path_to_vec(path, &mut self._submission_files)
    }

    pub fn add_base_path(&mut self, path: PathBuf) -> Result<(), Whatever> {
        Self::_add_path_to_vec(path, &mut self._base_files)
    }

    pub fn base_files(&self) -> impl Iterator<Item = &PathBuf> + '_ {
        self._base_files.iter()
    }

    pub fn submission_files<'a>(&'a self) -> impl Iterator<Item = &'a PathBuf> + 'a {
        self._submission_files.iter()
    }
}
