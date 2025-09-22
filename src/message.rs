use crate::types::MessageType;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

// GossipSub消息结构
#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub message_type: MessageType,
    pub message_id: String,
    pub timestamp: u64,
    pub from: Option<String>,
    pub to: Option<String>,
    pub topic: Option<String>,
    pub content: Option<Vec<u8>>,
    pub message_ids: Vec<String>, // 用于IHAVE/IWANT
}

impl GossipMessage {
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            message_id: Self::generate_id(),
            timestamp: Self::current_timestamp(),
            from: None,
            to: None,
            topic: None,
            content: None,
            message_ids: Vec::new(),
        }
    }

    pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }

    pub fn with_content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}