use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_own: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub name: String,
    pub avatar: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub unread_count: u32,
    pub messages: Vec<Message>,
}

impl Conversation {
    pub fn new(name: String, avatar: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            avatar,
            last_message: None,
            last_message_time: None,
            unread_count: 0,
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.last_message = Some(message.content.clone());
        self.last_message_time = Some(message.timestamp);
        if !message.is_own {
            self.unread_count += 1;
        }
        self.messages.push(message);
    }

    pub fn mark_as_read(&mut self) {
        self.unread_count = 0;
    }
}

pub fn generate_mock_data() -> Vec<Conversation> {
    let mut conversations = Vec::new();

    // Conversation 1: Alice
    let mut alice = Conversation::new("Alice".to_string(), "👩".to_string());
    alice.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Alice".to_string(),
        content: "Hey! How are you doing?".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(2),
        is_own: false,
    });
    alice.add_message(Message {
        id: Uuid::new_v4(),
        sender: "You".to_string(),
        content: "I'm good! How about you?".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(2) + chrono::Duration::minutes(5),
        is_own: true,
    });
    alice.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Alice".to_string(),
        content: "Doing great! Want to grab lunch tomorrow?".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(1),
        is_own: false,
    });
    conversations.push(alice);

    // Conversation 2: Bob
    let mut bob = Conversation::new("Bob".to_string(), "👨".to_string());
    bob.add_message(Message {
        id: Uuid::new_v4(),
        sender: "You".to_string(),
        content: "Did you finish the project?".to_string(),
        timestamp: Utc::now() - chrono::Duration::days(1),
        is_own: true,
    });
    bob.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Bob".to_string(),
        content: "Yes! Just submitted it.".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(20),
        is_own: false,
    });
    bob.mark_as_read();
    conversations.push(bob);

    // Conversation 3: Dev Team
    let mut dev_team = Conversation::new("Dev Team".to_string(), "💻".to_string());
    dev_team.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Sarah".to_string(),
        content: "Morning everyone! Daily standup in 10 minutes.".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(5),
        is_own: false,
    });
    dev_team.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Mike".to_string(),
        content: "I'll be there!".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(5) + chrono::Duration::minutes(2),
        is_own: false,
    });
    dev_team.add_message(Message {
        id: Uuid::new_v4(),
        sender: "You".to_string(),
        content: "On my way".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(5) + chrono::Duration::minutes(3),
        is_own: true,
    });
    dev_team.mark_as_read();
    conversations.push(dev_team);

    // Conversation 4: Carol
    let mut carol = Conversation::new("Carol".to_string(), "👩‍💼".to_string());
    carol.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Carol".to_string(),
        content: "Can you review my PR when you get a chance?".to_string(),
        timestamp: Utc::now() - chrono::Duration::minutes(30),
        is_own: false,
    });
    conversations.push(carol);

    // Conversation 5: Friends Group
    let mut friends = Conversation::new("Friends Group".to_string(), "🎉".to_string());
    friends.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Emma".to_string(),
        content: "Who's up for game night this Friday?".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(3),
        is_own: false,
    });
    friends.add_message(Message {
        id: Uuid::new_v4(),
        sender: "John".to_string(),
        content: "Count me in!".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(3) + chrono::Duration::minutes(5),
        is_own: false,
    });
    friends.add_message(Message {
        id: Uuid::new_v4(),
        sender: "You".to_string(),
        content: "Sounds fun! What time?".to_string(),
        timestamp: Utc::now() - chrono::Duration::hours(2) + chrono::Duration::minutes(30),
        is_own: true,
    });
    friends.add_message(Message {
        id: Uuid::new_v4(),
        sender: "Emma".to_string(),
        content: "How about 7 PM?".to_string(),
        timestamp: Utc::now() - chrono::Duration::minutes(45),
        is_own: false,
    });
    conversations.push(friends);

    conversations
}
