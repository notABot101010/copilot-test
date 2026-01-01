use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

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

#[derive(Debug, Deserialize)]
struct JoinRoomResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct SendMessageRequest {
    sender: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct SendMessageResponse {
    message_id: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct Message {
    id: String,
    room_id: String,
    sender: String,
    content: String,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct GetMessagesResponse {
    messages: Vec<Message>,
}

const BASE_URL: &str = "http://localhost:3000";

fn wait_for_server() {
    let client = Client::new();
    for _ in 0..30 {
        if client.get(&format!("{}/rooms", BASE_URL)).send().is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("Server did not start in time");
}

#[test]
fn test_end_to_end_two_clients() {
    wait_for_server();

    let client = Client::new();

    // Client 1: Create a room
    let create_req = CreateRoomRequest {
        name: "Test Room".to_string(),
        creator: "Alice".to_string(),
    };

    let create_resp: CreateRoomResponse = client
        .post(&format!("{}/rooms", BASE_URL))
        .json(&create_req)
        .send()
        .expect("Failed to create room")
        .json()
        .expect("Failed to parse create room response");

    assert_eq!(create_resp.name, "Test Room");
    let room_id = create_resp.room_id;

    // Client 2: Join the room
    let join_req = JoinRoomRequest {
        username: "Bob".to_string(),
    };

    let join_resp: JoinRoomResponse = client
        .post(&format!("{}/rooms/{}/join", BASE_URL, room_id))
        .json(&join_req)
        .send()
        .expect("Failed to join room")
        .json()
        .expect("Failed to parse join room response");

    assert!(join_resp.success);

    // Client 1: Send a message
    let msg_req = SendMessageRequest {
        sender: "Alice".to_string(),
        content: "Hello Bob!".to_string(),
    };

    let msg_resp: SendMessageResponse = client
        .post(&format!("{}/rooms/{}/messages", BASE_URL, room_id))
        .json(&msg_req)
        .send()
        .expect("Failed to send message")
        .json()
        .expect("Failed to parse send message response");

    assert!(!msg_resp.message_id.is_empty());

    // Client 2: Receive messages
    let messages_resp: GetMessagesResponse = client
        .get(&format!("{}/rooms/{}/messages", BASE_URL, room_id))
        .send()
        .expect("Failed to get messages")
        .json()
        .expect("Failed to parse messages response");

    assert_eq!(messages_resp.messages.len(), 1);
    assert_eq!(messages_resp.messages[0].sender, "Alice");
    assert_eq!(messages_resp.messages[0].content, "Hello Bob!");

    // Client 2: Send a reply
    let reply_req = SendMessageRequest {
        sender: "Bob".to_string(),
        content: "Hi Alice!".to_string(),
    };

    client
        .post(&format!("{}/rooms/{}/messages", BASE_URL, room_id))
        .json(&reply_req)
        .send()
        .expect("Failed to send reply")
        .json::<SendMessageResponse>()
        .expect("Failed to parse reply response");

    // Client 1: Receive all messages
    let all_messages: GetMessagesResponse = client
        .get(&format!("{}/rooms/{}/messages", BASE_URL, room_id))
        .send()
        .expect("Failed to get all messages")
        .json()
        .expect("Failed to parse all messages response");

    assert_eq!(all_messages.messages.len(), 2);
    assert_eq!(all_messages.messages[0].sender, "Alice");
    assert_eq!(all_messages.messages[0].content, "Hello Bob!");
    assert_eq!(all_messages.messages[1].sender, "Bob");
    assert_eq!(all_messages.messages[1].content, "Hi Alice!");

    println!("✓ All end-to-end tests passed!");
}
