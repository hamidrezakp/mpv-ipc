use serde_json::{self, Value};
use std::collections::HashMap;
use std::io::Error as IOError;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::net::UnixStream;

pub struct MPV {
    stream: UnixStream,
    requests_queue: Arc<Mutex<HashMap<_, _>>>,
}

pub trait FromValue: Sized {
    fn get_value(value: Value) -> Result<Self, Error>;
    fn as_string(&self) -> String;
}

impl MPV {
    pub async fn connect(path: &Path) -> Result<Self, IOError> {
        match UnixStream::connect(path).await {
            Ok(stream) => Ok(MPV(stream)),
            Err(error) => Err(error),
        }
    }

    pub async fn send_command<T: FromValue>(&self, cmd: Command) -> Result<T, Error> {
        //
    }
}

pub enum Command {
    Pause,
    Play,
    Seek(f64),
}

#[derive(Debug)]
pub enum Error {
    MpvError(String),
    JsonParseError(String),
    ConnectError(String),
    JsonContainsUnexptectedType,
    UnexpectedResult,
    UnexpectedValue,
    UnsupportedType,
    ValueDoesNotContainBool,
    ValueDoesNotContainF64,
    ValueDoesNotContainHashMap,
    ValueDoesNotContainPlaylist,
    ValueDoesNotContainString,
    ValueDoesNotContainUsize,
}
