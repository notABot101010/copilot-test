use qmetaobject::prelude::*;
use qmetaobject::SimpleListModel;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const BASE_URL: &str = "http://localhost:3000";

// Intermediate types for deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoomInfoRaw {
    id: String,
    name: String,
    #[serde(rename = "member_count")]
    member_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageRaw {
    id: String,
    #[serde(rename = "room_id")]
    room_id: String,
    sender: String,
    content: String,
    timestamp: i64,
}

// QML types
#[derive(Debug, Clone, Default)]
struct RoomInfo {
    pub id: QString,
    pub name: QString,
    pub member_count: i32,
}

impl qmetaobject::SimpleListItem for RoomInfo {
    fn get(&self, idx: i32) -> QVariant {
        match idx {
            0 => QVariant::from(&self.id),
            1 => QVariant::from(&self.name),
            2 => QVariant::from(self.member_count),
            _ => QVariant::default(),
        }
    }
    
    fn names() -> Vec<QByteArray> {
        vec![
            QByteArray::from("id"),
            QByteArray::from("name"),
            QByteArray::from("memberCount"),
        ]
    }
}

impl From<RoomInfoRaw> for RoomInfo {
    fn from(raw: RoomInfoRaw) -> Self {
        RoomInfo {
            id: QString::from(raw.id),
            name: QString::from(raw.name),
            member_count: raw.member_count,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Message {
    pub id: QString,
    pub room_id: QString,
    pub sender: QString,
    pub content: QString,
    pub timestamp: i64,
}

impl qmetaobject::SimpleListItem for Message {
    fn get(&self, idx: i32) -> QVariant {
        match idx {
            0 => QVariant::from(&self.id),
            1 => QVariant::from(&self.room_id),
            2 => QVariant::from(&self.sender),
            3 => QVariant::from(&self.content),
            4 => QVariant::from(self.timestamp),
            _ => QVariant::default(),
        }
    }
    
    fn names() -> Vec<QByteArray> {
        vec![
            QByteArray::from("id"),
            QByteArray::from("roomId"),
            QByteArray::from("sender"),
            QByteArray::from("content"),
            QByteArray::from("timestamp"),
        ]
    }
}

impl From<MessageRaw> for Message {
    fn from(raw: MessageRaw) -> Self {
        Message {
            id: QString::from(raw.id),
            room_id: QString::from(raw.room_id),
            sender: QString::from(raw.sender),
            content: QString::from(raw.content),
            timestamp: raw.timestamp,
        }
    }
}

#[derive(Debug, Serialize)]
struct CreateRoomRequest {
    name: String,
    creator: String,
}

#[derive(Debug, Deserialize)]
struct CreateRoomResponse {
    room_id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct JoinRoomRequest {
    username: String,
}

#[derive(Debug, Serialize)]
struct SendMessageRequest {
    sender: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GetMessagesResponse {
    messages: Vec<MessageRaw>,
}

#[derive(Debug, Deserialize)]
struct GetRoomsResponse {
    rooms: Vec<RoomInfoRaw>,
}

#[derive(QObject, Default)]
struct ChatApp {
    base: qt_base_class!(trait QObject),
    rooms: qt_property!(SimpleListModel<RoomInfo>; NOTIFY rooms_changed),
    rooms_changed: qt_signal!(),
    rooms_data: Vec<RoomInfoRaw>, // Keep the raw data for lookups
    messages: qt_property!(SimpleListModel<Message>; NOTIFY messages_changed),
    messages_changed: qt_signal!(),
    current_room_id: qt_property!(QString; NOTIFY current_room_changed),
    current_room_name: qt_property!(QString; NOTIFY current_room_changed),
    current_room_changed: qt_signal!(),
    status_text: qt_property!(QString; NOTIFY status_changed),
    status_changed: qt_signal!(),
    create_room: qt_method!(fn(&mut self, name: QString, creator: QString)),
    join_room: qt_method!(fn(&mut self, room_id: QString, username: QString)),
    send_message: qt_method!(fn(&mut self, room_id: QString, sender: QString, content: QString)),
    refresh_rooms: qt_method!(fn(&mut self)),
    poll_messages: qt_method!(fn(&mut self)),
    client: Arc<Mutex<reqwest::blocking::Client>>,
}

impl ChatApp {
    fn create_room(&mut self, name: QString, creator: QString) {
        let name_str = name.to_string();
        let creator_str = creator.to_string();
        
        self.status_text = QString::from("Creating room...");
        self.status_changed();
        
        let client = self.client.lock().unwrap();
        let req = CreateRoomRequest {
            name: name_str.clone(),
            creator: creator_str,
        };
        
        match client.post(&format!("{}/rooms", BASE_URL))
            .json(&req)
            .send()
        {
            Ok(resp) => {
                drop(client);
                match resp.json::<CreateRoomResponse>() {
                    Ok(_) => {
                        self.status_text = QString::from(format!("Room '{}' created!", name_str));
                        self.status_changed();
                        self.refresh_rooms();
                    }
                    Err(err) => {
                        self.status_text = QString::from(format!("Error creating room: {}", err));
                        self.status_changed();
                    }
                }
            }
            Err(err) => {
                self.status_text = QString::from(format!("Connection error: {}", err));
                self.status_changed();
            }
        }
    }
    
    fn join_room(&mut self, room_id: QString, username: QString) {
        let room_id_str = room_id.to_string();
        let username_str = username.to_string();
        
        self.status_text = QString::from("Joining room...");
        self.status_changed();
        
        let client = self.client.lock().unwrap();
        let req = JoinRoomRequest {
            username: username_str,
        };
        
        match client.post(&format!("{}/rooms/{}/join", BASE_URL, room_id_str))
            .json(&req)
            .send()
        {
            Ok(_) => {
                drop(client);
                // Find room name from cached data
                let room_name = self.rooms_data
                    .iter()
                    .find(|r| r.id == room_id_str)
                    .map(|r| QString::from(r.name.clone()))
                    .unwrap_or_else(|| QString::from("Unknown"));
                
                self.current_room_id = room_id;
                self.current_room_name = room_name;
                self.current_room_changed();
                
                // Clear messages and load new ones
                self.messages = Default::default();
                self.messages_changed();
                
                self.status_text = QString::from("Joined room!");
                self.status_changed();
                
                self.poll_messages();
            }
            Err(err) => {
                self.status_text = QString::from(format!("Error joining room: {}", err));
                self.status_changed();
            }
        }
    }
    
    fn send_message(&mut self, room_id: QString, sender: QString, content: QString) {
        let room_id_str = room_id.to_string();
        let sender_str = sender.to_string();
        let content_str = content.to_string();
        
        let client = self.client.lock().unwrap();
        let req = SendMessageRequest {
            sender: sender_str,
            content: content_str,
        };
        
        match client.post(&format!("{}/rooms/{}/messages", BASE_URL, room_id_str))
            .json(&req)
            .send()
        {
            Ok(_) => {
                drop(client);
                self.status_text = QString::from("Message sent!");
                self.status_changed();
                // Poll immediately to get the new message
                self.poll_messages();
            }
            Err(err) => {
                self.status_text = QString::from(format!("Error sending message: {}", err));
                self.status_changed();
            }
        }
    }
    
    fn refresh_rooms(&mut self) {
        self.status_text = QString::from("Refreshing rooms...");
        self.status_changed();
        
        let client = self.client.lock().unwrap();
        
        match client.get(&format!("{}/rooms", BASE_URL))
            .send()
        {
            Ok(resp) => {
                drop(client);
                match resp.json::<GetRoomsResponse>() {
                    Ok(data) => {
                        self.rooms_data = data.rooms.clone();
                        self.rooms = data.rooms.into_iter()
                            .map(RoomInfo::from)
                            .collect::<SimpleListModel<RoomInfo>>();
                        self.rooms_changed();
                        self.status_text = QString::from(format!("Found {} room(s)", self.rooms.row_count()));
                        self.status_changed();
                    }
                    Err(err) => {
                        self.status_text = QString::from(format!("Error parsing rooms: {}", err));
                        self.status_changed();
                    }
                }
            }
            Err(err) => {
                self.status_text = QString::from(format!("Connection error: {}", err));
                self.status_changed();
            }
        }
    }
    
    fn poll_messages(&mut self) {
        if self.current_room_id.to_string().is_empty() {
            return;
        }
        
        let room_id_str = self.current_room_id.to_string();
        let client = self.client.lock().unwrap();
        
        match client.get(&format!("{}/rooms/{}/messages", BASE_URL, room_id_str))
            .send()
        {
            Ok(resp) => {
                drop(client);
                match resp.json::<GetMessagesResponse>() {
                    Ok(data) => {
                        self.messages = data.messages.into_iter()
                            .map(Message::from)
                            .collect::<SimpleListModel<Message>>();
                        self.messages_changed();
                    }
                    Err(_) => {
                        // Silently ignore polling errors
                    }
                }
            }
            Err(_) => {
                // Silently ignore connection errors during polling
            }
        }
    }
}

fn main() {
    qml_register_type::<ChatApp>(cstr::cstr!("ChatApp"), 1, 0, cstr::cstr!("ChatApp"));
    
    let mut engine = QmlEngine::new();
    
    let qml_path = if std::path::Path::new("qt-chat/qml/main.qml").exists() {
        "qt-chat/qml/main.qml"
    } else if std::path::Path::new("qml/main.qml").exists() {
        "qml/main.qml"
    } else {
        eprintln!("Error: Could not find qml/main.qml");
        std::process::exit(1);
    };
    
    engine.load_file(qml_path.into());
    engine.exec();
}

