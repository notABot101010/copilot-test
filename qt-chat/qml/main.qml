import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import ChatApp 1.0

ApplicationWindow {
    id: window
    visible: true
    width: 800
    height: 600
    title: "Qt Chat Client"
    
    ChatApp {
        id: chatApp
    }
    
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10
        
        // Top bar with username and room controls
        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            
            Label {
                text: "Username:"
            }
            
            TextField {
                id: usernameField
                Layout.preferredWidth: 150
                placeholderText: "Enter username"
                text: "User" + Math.floor(Math.random() * 1000)
            }
            
            Item {
                Layout.fillWidth: true
            }
            
            Button {
                text: "Refresh Rooms"
                onClicked: chatApp.refreshRooms()
            }
            
            TextField {
                id: newRoomField
                Layout.preferredWidth: 200
                placeholderText: "New room name"
            }
            
            Button {
                text: "Create Room"
                enabled: newRoomField.text.length > 0 && usernameField.text.length > 0
                onClicked: {
                    chatApp.createRoom(newRoomField.text, usernameField.text)
                    newRoomField.text = ""
                }
            }
        }
        
        // Main content area
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 10
            
            // Room list
            GroupBox {
                title: "Rooms"
                Layout.preferredWidth: 250
                Layout.fillHeight: true
                
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 5
                    
                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        
                        ListView {
                            id: roomListView
                            model: chatApp.rooms
                            delegate: Button {
                                text: modelData.name + " (" + modelData.memberCount + " members)"
                                width: parent ? parent.width - 10 : 200
                                onClicked: {
                                    if (usernameField.text.length > 0) {
                                        chatApp.joinRoom(modelData.id, usernameField.text)
                                    }
                                }
                            }
                            spacing: 5
                        }
                    }
                }
            }
            
            // Chat area
            GroupBox {
                title: chatApp.currentRoomName.length > 0 ? "Chat: " + chatApp.currentRoomName : "Select a room"
                Layout.fillWidth: true
                Layout.fillHeight: true
                
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 10
                    
                    // Messages area
                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        
                        ListView {
                            id: messageListView
                            model: chatApp.messages
                            delegate: Rectangle {
                                width: parent ? parent.width - 10 : 400
                                height: messageColumn.height + 10
                                color: modelData.sender === usernameField.text ? "#e3f2fd" : "#f5f5f5"
                                radius: 5
                                
                                ColumnLayout {
                                    id: messageColumn
                                    anchors.fill: parent
                                    anchors.margins: 5
                                    spacing: 2
                                    
                                    Label {
                                        text: modelData.sender
                                        font.bold: true
                                        font.pixelSize: 12
                                    }
                                    
                                    Label {
                                        text: modelData.content
                                        wrapMode: Text.WordWrap
                                        Layout.fillWidth: true
                                    }
                                    
                                    Label {
                                        text: new Date(modelData.timestamp * 1000).toLocaleTimeString()
                                        font.pixelSize: 10
                                        color: "#666"
                                    }
                                }
                            }
                            spacing: 5
                            
                            onCountChanged: {
                                if (count > 0) {
                                    positionViewAtEnd()
                                }
                            }
                        }
                    }
                    
                    // Message input
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 10
                        
                        TextField {
                            id: messageField
                            Layout.fillWidth: true
                            placeholderText: "Type your message..."
                            enabled: chatApp.currentRoomId.length > 0
                            onAccepted: sendButton.clicked()
                        }
                        
                        Button {
                            id: sendButton
                            text: "Send"
                            enabled: messageField.text.length > 0 && chatApp.currentRoomId.length > 0
                            onClicked: {
                                chatApp.sendMessage(chatApp.currentRoomId, usernameField.text, messageField.text)
                                messageField.text = ""
                            }
                        }
                    }
                }
            }
        }
        
        // Status bar
        Label {
            text: chatApp.statusText
            Layout.fillWidth: true
            color: "#666"
            font.pixelSize: 10
        }
    }
    
    Timer {
        interval: 2000
        running: chatApp.currentRoomId.length > 0
        repeat: true
        onTriggered: chatApp.pollMessages()
    }
}
