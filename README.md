# rusty-gst-qt

# Overview

Experiments around usage of GStreamer and Qt in Rust. both are extremely powerful frameworks, used by me at my day job with C++.
This repo is my attempt to solve a question I was posing for a while- is Rust ready for creating large-scale desktop applications? Although more work is needed, preliminary results are exciting and encourage further exploration.


Tested on Ubuntu 20.04, with GStreamer 1.16 & Qt 5.12.

## Functionality 
For now, the demo involves 2 processes;
- launcher with QML GUI
- child process, running GStreamer.

The demo allows selecting a video file to playback, starting the process, and controlling the playback state (playing/paused). A notification from child process indicates whether the media is currently playing. 
GUI is extremely basic, no effort was made so far to make it pretty. 

## Roadmap
- [ ] use proper IPC (ipc-channel, or tokio components) instead of OSC
- [ ] fix a nasty hack involving using unsafe code (sharing state in the launcher)
- [ ] add more functionality to GStreamer part - implementing signal handlers, pipeline creation from elements etc. Porting existing functionality from my C application might be a good start: [gstreamer-video-streaming](https://github.com/klochowicz/gstreamer-video-streaming)
- [ ] create a nice GUI in QML (using Qt Design Studio)

[![Built with Spacemacs](https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg)](http://spacemacs.org)
