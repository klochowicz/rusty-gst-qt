import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Dialogs 1.3

Item {
    id: chooser

    property alias path: pathContent.text
    property alias label: pathLabel.text

    width: 400
    height: 30

    Row {
        anchors.fill: parent
        spacing: 10

        Label {
            id: pathLabel

            height: parent.height
        }

        Button {
            text: "Choose File"
            onClicked: {
                fileDialog.visible = !fileDialog.visible;
            }
        }

        TextEdit {
            id: pathContent

            height: parent.height
        }


        FileDialog {
            id: fileDialog

            title: "Please choose a file for " + chooser.label
            visible: false
            folder: shortcuts.home
            onAccepted: {
                console.log("You chose: " + fileDialog.fileUrls);
                var file_path = fileDialog.fileUrl.toString();
                // remove prefixed "file:///"
                file_path = file_path.replace(/^(file:\/{2})/, "");
                // unescape html codes like '%23' for '#'
                chooser.path = decodeURIComponent(file_path);
            }
            onRejected: {
                console.log("Canceled");
                visible = false;
            }
        }

    }

}
