// based on https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/blob/master/examples/src/bin/glib-futures.rs
use gstreamer::prelude::*;

use async_osc::{prelude::*, OscPacket, OscSocket};
use async_std::stream::StreamExt;
use clap::{AppSettings, Clap};

static OSC_SOCKET: &str = "localhost:0";
static OSC_LISTEN: &str = "localhost:5050";
static OSC_SEND: &str = "localhost:5051";

/// Listen to commands received via OSC to steer the GStreamer pipeline
async fn command_handler(pipeline: gstreamer::Element) {
    let mut socket = OscSocket::bind(OSC_LISTEN).await.unwrap();

    // Listen for incoming packets .
    while let Some(packet) = socket.next().await {
        let (packet, _) = packet.unwrap();
        match packet {
            OscPacket::Bundle(_) => {}
            OscPacket::Message(message) => match message.as_tuple() {
                ("/command/play", _) => {
                    println!("Play requested");
                    pipeline
                        .set_state(gstreamer::State::Playing)
                        .expect("Unable to set the pipeline to the `Playing` state");
                }
                ("/command/pause", _) => {
                    println!("Pause requested");
                    pipeline
                        .set_state(gstreamer::State::Paused)
                        .expect("Unable to set the pipeline to the `Paused` state");
                }
                _ => {}
            },
        }
    }
}

/// Handle GStreamer messages that show up on the mainloop by forwarding them
/// via OSC to the parent process
async fn message_handler(loop_: glib::MainLoop, bus: gstreamer::Bus) {
    let mut messages = bus.stream();

    let socket = OscSocket::bind(OSC_SOCKET)
        .await
        .expect("Cannot create OSC socket");
    socket
        .connect(OSC_SEND)
        .await
        .expect("Cannot connect to socket");

    while let Some(msg) = messages.next().await {
        use gstreamer::MessageView;

        // Determine whether we want to quit: on EOS or error message
        // we quit, otherwise simply continue.
        match msg.view() {
            MessageView::Eos(..) => {
                println!("Received EOS. Exiting the main loop.");
                socket
                    .send(("/state/eos", (1i32,)))
                    .await
                    .expect("Can't send data over socket");
                loop_.quit();
            }
            MessageView::StateChanged(state) => {
                if state
                    .get_src()
                    .expect("No source of message")
                    .dynamic_cast::<gstreamer::Pipeline>()
                    .is_ok()
                {
                    println!("Sending an OSC message from child process");
                    match state.get_current() {
                        gstreamer::State::Playing => {
                            socket
                                .send(("/state/playing", (1i32,)))
                                .await
                                .expect("Can't send data over socket");
                        }
                        gstreamer::State::Paused => {
                            socket
                                .send(("/state/paused", (1i32,)))
                                .await
                                .expect("Can't send data over socket");
                        }
                        _ => {}
                    }
                }
            }
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
                socket
                    .send(("/state/error", (1i32,)))
                    .await
                    .expect("Can't send data over socket");
                loop_.quit();
            }
            _ => (),
        }
    }
}

fn make_pipeline_string(opts: &Opts) -> String {
    "playbin uri=file://".to_string() + &opts.video_path
}

fn gstreamer_main(opts: &Opts) {
    // Get the default main context and make it also the thread default, then create
    // a main loop for it
    let ctx = glib::MainContext::default();
    ctx.push_thread_default();
    let loop_ = glib::MainLoop::new(Some(&ctx), false);

    gstreamer::init().unwrap();

    // Create a pipeline from the launch-syntax given on the cli.
    let pipeline_string = make_pipeline_string(&opts);
    // let pipeline_string = "videotestsrc ! autovideosink";
    println!("pipeline string: {}", pipeline_string);

    let pipeline = gstreamer::parse_launch(&pipeline_string).expect("Invalid pipeline syntax");
    let bus = pipeline.get_bus().unwrap();

    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Unable to set the pipeline to the `Paused` state");

    // Spawn handler streams
    ctx.spawn_local(command_handler(pipeline.clone()));
    ctx.spawn_local(message_handler(loop_.clone(), bus));

    loop_.run();

    pipeline
        .set_state(gstreamer::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");

    ctx.pop_thread_default();
}

/// Describe CLI arguments
#[derive(Clap)]
#[clap(version = "0.1", author = "Mariusz K. <mariusz@klochowicz.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short = 'f', long)]
    video_path: String,
    // TODO: Add support for specifying width and height
    #[clap(short, long, default_value = "1920")]
    _width: u32,
    #[clap(short, long, default_value = "1080")]
    _height: u32,
}

fn main() {
    let opts: Opts = Opts::parse();
    gstreamer_main(&opts);
}
