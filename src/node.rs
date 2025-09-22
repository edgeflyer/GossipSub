use std::collections::{HashMap, HashSet};
use crate::message::GossipMessage;
use crate::types::GossipSubConfig;
// GossipSub节点
pub struct GossipSubNode {
    pub node_id: String,
    pub peers: HashMap<String, String>, // peerId -> peer连接信息
    pub topics: HashSet<String>, // 订阅的主题
    pub mesh: HashMap<String, HashSet<String>>, // topic -> Set(peers)
    pub fanout: HashMap<String, HashSet<String>>, // fanout网络
    pub message_cache: HashMap<String, GossipMessage>, // messageId -> message
    pub seen_messages: HashSet<String>, // 已见过的消息ID
    pub config: GossipSubConfig,
}

impl GossipSubNode {
    pub fn new(node_id: String) -> Self {
        println!("GossipSub节点 {} 已创建", node_id);

        Self {
            node_id,
            peers: HashMap::new(),
            topics: HashSet::new(),
            mesh: HashMap::new(),
            fanout: HashMap::new(),
            message_cache: HashMap::new(),
            seen_messages: HashSet::new(),
            config: GossipSubConfig::default(),
        }
    }

    // 添加对等节点连接
    pub fn add_peer(&mut self, peer_id:String, connection_info: String) {
        self.peers.insert(peer_id.clone(), connection_info);
        println!("节点 {} 连接到对等节点 {}", self.node_id, peer_id);
    }

    // 订阅主题
    pub fn subscribe(&mut self, topic: String) {
        if !self.topics.contains(&topic) {
            self.topics.insert(topic.clone());
            println!("节点 {} 订阅主题: {}", self.node_id, topic);

            // 初始化该主题的mesh网络
            self.initialize_mesh(&topic);
        }
    }

    // 初始化主题的mesh网络
    fn initialize_mesh(&mut self, topic: &str) {
        if !self.mesh.contains_key(topic) {
            self.mesh.insert(topic.to_string(), HashSet::new());
        }

        // 获取所有对等节点（这里简化处理，实际应该检查对等节点是否订阅了相同的主题)
        let available_peers: Vec<String> = self.peers.keys().cloned().collect();

        let mesh_peers = self.mesh.get_mut(topic).unwrap();
        let needed = std::cmp::min(self.config.mesh_size, available_peers.len());

        // 随机选择节点加入mesh(这里简化为顺序选择)
        for(i, peer_id) in available_peers.iter().enumerate() {
            if i >= needed {
                break;
            }
            mesh_peers.insert(peer_id.clone());
        }

        println!("节点 {} 在主题 {} 的mesh中有 {} 个节点", self.node_id, topic, mesh_peers.len());
    }

    // 获取mesh中的节点数量
    pub fn get_mesh_size(&self, topic: &str) -> usize {
        self.mesh.get(topic).map_or(0, |peers| peers.len())
    }

    // 检查是否在某个主题的mesh中
    pub fn is_in_mesh(&self, topic: &str, peer_id: &str) -> bool {
        self.mesh.get(topic)
            .map_or(false, |peers| peers.contains(peer_id))
    }
}