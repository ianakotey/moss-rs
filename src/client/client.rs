use std::{
    borrow::Cow,
    cell::RefCell,
    fs::File,
    io::{self, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    process::exit,
};

use extend::ext;
use lazy_static::lazy_static;
use regex::Regex;
use snafu::{prelude::*, Whatever};

use crate::prelude::MossConfig;

#[derive(Debug)]
pub struct MossClient<S: ToSocketAddrs> {
    server: RefCell<TcpStream>,
    config: MossConfig<S>,
}

impl<S: ToSocketAddrs> MossClient<S> {
    pub fn new<U: ToString>(server: S, user_id: U) -> io::Result<Self> {
        MossConfig::new(user_id.to_string(), server).try_into()
    }

    pub fn send(mut self) -> Result<String, Whatever> {
        self._send_headers()?;
        self._upload_base_files()?;
        self._upload_submission_files()?;
        self._query_server()?;

        self.server
            .borrow_mut()
            ._read_string_512()
            .whatever_context("Unable to read Url from server")
    }

    pub fn add_base_file<P: AsRef<str>>(&mut self, path: P) -> Result<(), Whatever> {
        self.config.add_base_file(&path).map(|_| ())
    }

    pub fn add_file<P: AsRef<str>>(&mut self, path: P) -> Result<(), Whatever> {
        self.config.add_file(&path).map(|_| ())
    }

    fn _send_file<P: AsRef<Path>>(&self, file: P, file_index: usize) -> Result<(), Whatever> {
        if file.as_ref().exists() {
            print!("Uploaded {:?}.... ", file.as_ref());

            let f = File::open(&file).whatever_context("Could not open file")?;
            let mut file_buffer = Vec::new();

            let _ = BufReader::new(f)
                .read_to_end(&mut file_buffer)
                .whatever_context("Could not read data from file")?;

            //TODO to be replaced once path::absolute is stabilized
            let canonicallized_path = file.as_ref().canonicalize().with_whatever_context(|_| {
                format!("File does not exist: {}", file.as_ref().to_string_lossy())
            })?;
            let file_path = canonicallized_path
                .to_str()
                .whatever_context("Invalid / Non UTF-8 file name")?;

            let file_path = match std::env::consts::OS {
                "windows" => {
                    lazy_static! {
                        static ref PATH_RE: Regex = Regex::new(r"^(?:\\\\\?\\)*(\w:)").unwrap();
                    }
                    Cow::from(PATH_RE.replace(file_path, "").replace(r"\", "/"))
                }
                _ => Cow::from(file_path),
            };

            let display_name = match self.config.transform() {
                Some(pattern) => {
                    let re = Regex::new(pattern.as_ref()).with_whatever_context(|_| {
                        format!("Invalid regex expression provided: {}", pattern)
                    })?;

                    if let Some(val) = re.captures(&file_path) {
                        Cow::from(
                            val.iter()
                                .skip(1)
                                .flatten()
                                .map(|c| c.as_str())
                                .intersperse("/")
                                .collect::<String>(),
                        )
                    } else {
                        file_path
                    }
                }
                None => file_path,
            };

            self.server
                .borrow_mut()
                .write(
                    format!(
                        "file {} {} {} {}\n",
                        file_index,
                        self.config.language(),
                        file_buffer.len(),
                        display_name.replace(" ", "_").trim_end_matches('/')
                    )
                    .as_bytes(),
                )
                .whatever_context("Unable to write file description")?;

            self.server
                .borrow_mut()
                .write(file_buffer.as_slice())
                .whatever_context("Unable to write file description")?;

            println!("done.");
            Ok(())
        } else {
            whatever!("File does not exist");
        }
    }

    fn _send_headers(&self) -> Result<(), Whatever> {
        let _ = self
            .server
            .borrow_mut()
            .write(format!("moss {}\n", self.config.user_id()).as_bytes())
            .whatever_context("Could not authenticate with Moss")?;
        let _ = self
            .server
            .borrow_mut()
            .write(
                format!(
                    "directory {}\n",
                    self.config.use_directory_mode().as_moss_option()
                )
                .as_bytes(),
            )
            .whatever_context("Error sending directory information to Moss server")?;
        let _ = self
            .server
            .borrow_mut()
            .write(
                format!(
                    "X {}\n",
                    self.config.use_experimental_mode().as_moss_option()
                )
                .as_bytes(),
            )
            .whatever_context("Error sending experimental information to Moss server")?;
        let _ = self
            .server
            .borrow_mut()
            .write(format!("maxmatches {}\n", self.config.max_ignore_threshold()).as_bytes())
            .whatever_context("Error sending match information to Moss server")?;
        let _ = self
            .server
            .borrow_mut()
            .write(format!("show {}\n", self.config.max_matches_displayed()).as_bytes())
            .whatever_context("Error sending display information to Moss server")?;
        let _ = self
            .server
            .borrow_mut()
            .write(format!("language {}\n", self.config.language()).as_bytes())
            .whatever_context("Error sending language information to Moss server")?;

        let header_response = self
            .server
            .borrow_mut()
            ._read_string_512()
            .whatever_context("Unable to receive server's header response")?;

        if !header_response.trim().eq_ignore_ascii_case("yes") {
            println!("Unsupported language: {}", self.config.language());
            exit(1);
        }

        Ok(())
    }

    fn _upload_base_files(&self) -> Result<(), Whatever> {
        // dbg!(format!("Uploading the following base files: {:?}", self.config.base_files().collect::<Vec<_>>()));
        for file in self.config.base_files() {
            self._send_file(file, 0)?;
        }
        Ok(())
    }

    fn _upload_submission_files(&self) -> Result<(), Whatever> {
        for (file, index) in self.config.submission_files().zip(1..) {
            self._send_file(file, index as usize)?;
        }
        Ok(())
    }

    fn _query_server(&mut self) -> Result<(), Whatever> {
        // FIXME: Probable problem area. Might need to manually add quotes
        self.server
            .borrow_mut()
            .write(format!("query 0 {}\n", self.config.comment()).as_bytes())
            .whatever_context("Could not send query to server")?;
        println!("Query submitted.  Waiting for the server's response.\n");
        Ok(())
    }
}

impl<S: ToSocketAddrs> TryFrom<MossConfig<S>> for MossClient<S> {
    type Error = io::Error;
    fn try_from(config: MossConfig<S>) -> Result<Self, Self::Error> {
        Ok(MossClient {
            server: RefCell::new(TcpStream::connect(&config.server_address())?),
            config,
        })
    }
}

#[doc(hidden)]
#[ext]
impl TcpStream {
    /// Read a string from the socket using a 512 byte buffer
    /// This method is for internal use only.
    fn _read_string_512(&mut self) -> Result<String, io::Error> {
        let mut byte_array = [32; 512];

        match self.read(&mut byte_array) {
            Ok(_bytes_read) => Ok(String::from_utf8_lossy(&byte_array).trim().to_string()),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok(String::new()),
            Err(err) => Err(err),
        }
    }
}

#[doc(hidden)]
#[ext]
impl bool {
    #[inline(always)]
    fn as_moss_option(&self) -> &'static str {
        if *self == true {
            "1"
        } else {
            "0"
        }
    }
}
