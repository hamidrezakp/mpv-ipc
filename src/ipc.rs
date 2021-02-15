extern crate log;

use log::{debug, warn};
use serde_json::{self, Value};
use std::fmt::{self, Display};
use std::io::prelude::*;
use std::io::{BufReader, Read};
use std::os::unix::net::UnixStream;

pub enum Event {
    Pause(f64),
    Play(f64),
    Seek(f64),
    Unimplemented,
}

pub struct Mpv {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
    name: String,
}

impl Mpv {
    pub fn connect(socket: &str) -> Result<Mpv, ErrorCode> {
        match UnixStream::connect(socket) {
            Ok(stream) => {
                let cloned_stream = stream.try_clone().expect("cloning UnixStream");
                return Ok(Mpv {
                    stream,
                    reader: BufReader::new(cloned_stream),
                    name: String::from(socket),
                });
            }
            Err(internal_error) => Err(ErrorCode::ConnectError(internal_error.to_string())),
        }
    }

    pub fn disconnect(&self) {
        let mut stream = &self.stream;
        stream
            .shutdown(std::net::Shutdown::Both)
            .expect("socket disconnect");
        let mut buffer = [0; 32];
        for _ in 0..stream.bytes().count() {
            stream.read(&mut buffer[..]).unwrap();
        }
    }

    pub fn get_stream_ref(&self) -> &UnixStream {
        &self.stream
    }

    fn run_command(&self, command: &str, args: &[&str]) -> Result<(), ErrorCode> {
        let mut ipc_string = format!("{{ \"command\": [\"{}\"", command);
        if args.len() > 0 {
            for arg in args {
                ipc_string.push_str(&format!(", \"{}\"", arg));
            }
        }
        ipc_string.push_str("] }\n");
        ipc_string = ipc_string;
        match serde_json::from_str::<Value>(&self.send_command_sync(&ipc_string)) {
            Ok(feedback) => {
                if let Value::String(ref error) = feedback["error"] {
                    if error == "success" {
                        Ok(())
                    } else {
                        Err(ErrorCode::MpvError(error.to_string()))
                    }
                } else {
                    Err(ErrorCode::UnexpectedResult)
                }
            }
            Err(why) => Err(ErrorCode::JsonParseError(why.to_string())),
        }
    }

    fn send_command_sync(&self, command: &str) -> String {
        let mut stream = &self.stream;
        match stream.write_all(command.as_bytes()) {
            Err(why) => panic!("Error: Could not write to socket: {}", why),
            Ok(_) => {
                debug!("Command: {}", command.trim_end());
                let mut response = String::new();
                {
                    let mut reader = BufReader::new(stream);
                    while !response.contains("\"error\":") {
                        response.clear();
                        reader.read_line(&mut response).unwrap();
                    }
                }
                debug!("Response: {}", response.trim_end());
                response
            }
        }
    }

    pub fn listen(&mut self) -> Result<Event, ErrorCode> {
        let mut response = String::new();
        self.reader.read_line(&mut response).unwrap();
        response = response.trim_end().to_string();
        debug!("Event: {}", response);
        match serde_json::from_str::<Value>(&response) {
            Ok(e) => {
                if let Value::String(ref name) = e["event"] {
                    let event: Event;
                    match name.as_str() {
                        "seek" => {
                            let pos = self.get_current_pos().unwrap();
                            event = Event::Seek(pos);
                        }
                        "pause" => {
                            let pos = self.get_current_pos().unwrap();
                            event = Event::Pause(pos);
                        }
                        "unpause" => {
                            let pos = self.get_current_pos().unwrap();
                            event = Event::Play(pos);
                        }
                        _ => {
                            event = Event::Unimplemented;
                        }
                    };
                    return Ok(event);
                }
            }
            Err(why) => return Err(ErrorCode::JsonParseError(why.to_string())),
        }
        unreachable!();
    }

    pub fn play(&self) -> Result<(), ErrorCode> {
        //TODO
        unimplemented!()
    }

    pub fn pause(&self) -> Result<(), ErrorCode> {
        //TODO
        unimplemented!()
    }

    pub fn seek(&self, position: f64) -> Result<(), ErrorCode> {
        //TODO
        unimplemented!()
    }

    fn get_current_pos(&self) -> Result<f64, ErrorCode> {
        let ipc_string = r#"{{ "command": ["get_property","time-pos"] }}\n"#;

        match serde_json::from_str::<Value>(&self.send_command_sync(&ipc_string)) {
            Ok(val) => Ok(val.as_f64().unwrap()),
            Err(why) => Err(ErrorCode::JsonParseError(why.to_string())),
        }
    }
}

impl Drop for Mpv {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[derive(Debug)]
pub enum ErrorCode {
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

pub enum MpvCommand {
    Pause,
    play,
    Seek(f64),
}
