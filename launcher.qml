import LaunchGui 1.0
import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Dialogs 1.3
import "qrc:/qml"

ApplicationWindow {
    visible: true
    color: "#fcc2c2"
    title: "Rust GStreamer Qt player dashboard"
    width: 800
    height: 200

    MainGui {
        id: launcher
        video: chooser.path
    }

    // FIXME: Use a fancier method instead of continuously polling for state update
    Timer {
        interval: 200; running: true; repeat: true
        onTriggered: launcher.check_playing();
    }

    Rectangle {
        anchors.fill: parent

        FileChooser {
            id: chooser

            y: 50
            label: "Video"
            height: 30
        }

        Row {
            spacing: 20
            y: 100

            Button {
                text: "Launch"
                action: action_start
            }

            Button {
                text: "Stop"
                action: action_stop
            }

            Button {
                text: "Play"
                action: action_play
            }

            Button {
                text: "Pause"
                action: action_pause
            }

            Label {
                id: readout
                visible: launcher.playing && launcher.started;
                text: "It is playing"

            }
        }

        // Non-visible elements 
        
        Action {
            id: action_start

            enabled: !launcher.started
            onTriggered: {
                launcher.start();
            }
        }

        Action {
            id: action_stop

            enabled: launcher.started
            onTriggered: {
                launcher.stop();
            }
        }

        Action {
            id: action_play

            enabled: launcher.started
            onTriggered: {
                launcher.play();
            }
        }

        Action {
            id: action_pause

            enabled: launcher.started
            onTriggered: {
                launcher.pause();
            }
        }


    }

    header: ToolBar {
        Label {
            anchors.fill: parent
            text: qsTr("GStreamer Twitch Video Streamer")
            font.pixelSize: 30
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }

    }

}
