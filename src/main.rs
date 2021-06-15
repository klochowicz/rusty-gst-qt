#[macro_use]
extern crate lazy_static;

use async_osc::{prelude::*, Error, OscPacket, OscSocket};
use async_std::stream::StreamExt;
use core::panic;
use cstr::cstr;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use qmetaobject::prelude::*;

static OSC_SOCKET: &str = "localhost:0";
static OSC_ADDRESS: &str = "localhost:5050";

qrc!(my_resource,
     "qml" {
         "launcher.qml",
         "FileChooser.qml"
     },
);

pub fn launch_async_runner(video_path: &str) -> Child {
    let handle = Command::new("target/debug/runner-async")
        .arg("--video-path")
        .arg(video_path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");
    handle
}

#[derive(QObject, Default)]
struct LaunchGui {
    // Specify the base class with the qt_base_class macro
    base: qt_base_class!(trait QObject),
    video: qt_property!(QString; NOTIFY video_changed),
    video_changed: qt_signal!(),
    started: qt_property!(bool; NOTIFY started_changed),
    started_changed: qt_signal!(),
    playing: qt_property!(bool; NOTIFY playing_changed),
    playing_changed: qt_signal!(),
    start: qt_method!(fn(&self)),
    stop: qt_method!(fn(&self)),
    play: qt_method!(fn(&self)),
    pause: qt_method!(fn(&self)),
    check_playing: qt_method!(fn(&self)),
    // Non-qt data
    handle: Option<Child>,
}

impl LaunchGui {
    fn update_running_state(&mut self) {
        self.started = self.handle.is_some();
        self.started_changed();
    }

    fn start(&mut self) {
        self.handle = {
            if self.video.to_string().len() != 0 {
                Some(launch_async_runner(&self.video.to_string()))
            } else {
                eprintln!("No video file selected. Attempting to launch in debug mode");
                let file_name = "sample.mp4";
                // TODO: Use a crate to find out about user $HOME directory
                let file_path = "/home/mariuszk/".to_string() + file_name;

                if !std::path::Path::new(&file_path).exists() {
                    panic!("Debug mode unavailable. You need to place a file 'sample.mp4' in your HOME directory");
                }
                Some(launch_async_runner(&file_path))
            }
        };
        self.update_running_state();
    }

    fn check_playing(&mut self) {
        let state_data = STATE.lock().unwrap();
        let playing = is_playing(&*state_data);
        if self.playing != playing {
            self.playing = playing;
            self.playing_changed();
        }
    }

    fn stop(&mut self) {
        match &mut self.handle {
            Some(handle) => handle.kill().expect("Couldn't kill the process"),
            None => panic!("Handle should be available"),
        }
        self.handle = None;
        self.update_running_state();
    }

    fn play(&self) {
        async_std::task::spawn(async move {
            let socket = OscSocket::bind(OSC_SOCKET).await?;
            socket.connect(OSC_ADDRESS).await?;
            socket.send(("/command/play", (1i32,))).await?;
            Ok::<(), Error>(())
        });
    }

    fn pause(&self) {
        async_std::task::spawn(async move {
            let socket = OscSocket::bind(OSC_SOCKET).await?;
            socket.connect(OSC_ADDRESS).await?;
            socket.send(("/command/pause", (1i32,))).await?;
            Ok::<(), Error>(())
        });
    }
}

async fn command_handler() {
    let mut socket = OscSocket::bind("localhost:5051").await.unwrap();

    // Listen for incoming packets .
    while let Some(packet) = socket.next().await {
        let (packet, _) = packet.unwrap();
        match packet {
            OscPacket::Bundle(_) => {}
            OscPacket::Message(message) => match message.as_tuple() {
                ("/state/playing", _) => {
                    let mut state_data = STATE.lock().unwrap();
                    *state_data = PipelineState::Playing;
                }
                ("/state/paused", _) => {
                    let mut state_data = STATE.lock().unwrap();
                    *state_data = PipelineState::Paused;
                }
                ("/state/error", _) => {
                    let mut state_data = STATE.lock().unwrap();
                    *state_data = PipelineState::Error;
                }
                ("/state/eos", _) => {
                    let mut state_data = STATE.lock().unwrap();
                    *state_data = PipelineState::EndOfStream;
                }
                _ => {
                    eprintln!("Received unrecognised OSC message: {:?}", message);
                }
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum PipelineState {
    Unknown,
    Playing,
    Paused,
    Error,
    EndOfStream,
}

fn is_playing(state: &PipelineState) -> bool {
    *state == PipelineState::Playing
}

// FIXME: Find a way of using STATE inside Qt object instead of polling
lazy_static! {
    static ref STATE: Arc<Mutex<PipelineState>> = Arc::new(Mutex::new(PipelineState::Unknown));
}

fn main() {
    qmetaobject::future::execute_async(command_handler());

    my_resource();
    qml_register_type::<LaunchGui>(cstr!("LaunchGui"), 1, 0, cstr!("MainGui"));
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/qml/launcher.qml".into());
    engine.exec();
}
