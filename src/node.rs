use crate::message::{self, GossipMessage};
use crate::types::{GossipSubConfig, MessageType};
use std::collections::{HashMap, HashSet};
// GossipSub节点
pub struct GossipSubNode {
    pub node_id: String,
    pub peers: HashMap<String, String>, // peerId -> peer连接信息
    pub topics: HashSet<String>,        // 订阅的主题
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

    // 发布消息到指定主题
    pub fn publish(&mut self, topic: &str, content: Vec<u8>) -> Result<String, String> {
        if !self.topics.contains(topic) {
            return Err(format!("节点 {} 未订阅主题 {}", self.node_id, topic));
        }

        let message = GossipMessage::new(MessageType::Publish)
            .with_topic(topic.to_string())
            .with_content(content)
            .with_from(self.node_id.clone());
        let message_id = message.message_id.clone();

        // 缓存消息
        self.message_cache
            .insert(message_id.clone(), message.clone());
        self.seen_messages.insert(message_id.clone());

        println!(
            "节点 {} 发布消息到主题 {}: ID={}",
            self.node_id, topic, message_id
        );

        // 转发消息给mesh中的节点
        self.forward_to_mesh(topic, &message)?;

        // 如果没有mesh节点，使用fanout
        if self.get_mesh_size(topic) == 0 {
            self.forward_to_fanout(topic, &message)?;
        }

        Ok(message_id)
    }

    // 转发消息给mesh网络中的节点
    fn forward_to_mesh(&self, topic: &str, message: &GossipMessage) -> Result<(), String> {
        if let Some(mesh_peers) = self.mesh.get(topic) {
            for peer_id in mesh_peers {
                self.send_message_to_peer(peer_id, message)?;
            }
            println!(
                "节点 {} 向mesh中的 {} 个节点转发了消息",
                self.node_id,
                mesh_peers.len()
            );
        }

        Ok(())
    }

    // 转发消息给fanout网络中的节点
    fn forward_to_fanout(&mut self, topic: &str, message: &GossipMessage) -> Result<(), String> {
        // 如果fanout不存在创造一个
        if !self.fanout.contains_key(topic) {
            self.fanout.insert(topic.to_string(), HashSet::new());

            // 从非mesh中的peer中随机选择fanout节点
            let available_peers: Vec<String> = self
                .peers
                .keys()
                .filter(|&peer_id| !self.is_in_mesh(topic, peer_id))
                .cloned()
                .collect();

            let fanout_peers = self.fanout.get_mut(topic).unwrap();
            let needed = std::cmp::min(self.config.gossip_size, available_peers.len());

            for (i, peer_id) in available_peers.iter().enumerate() {
                if i >= needed {
                    break;
                }
                fanout_peers.insert(peer_id.clone());
            }
        }

        // 转发消息给fanout节点
        if let Some(fanout_peers) = self.fanout.get(topic) {
            for peer_id in fanout_peers {
                self.send_message_to_peer(peer_id, message)?;
            }
            println!(
                "节点 {} 向fanout中的 {} 个节点转发了消息",
                self.node_id,
                fanout_peers.len()
            );
        }

        Ok(())
    }

    // 发送消息给指定的对等节点
    fn send_message_to_peer(&self, peer_id: &str, message: &GossipMessage) -> Result<(), String> {
        // 这里模拟发送消息的过程
        // 在实际实现中，这里会通过网络发送消息
        println!(
            "  {} -> {}: 发送 {:?} 消息 (ID: {})",
            self.node_id, peer_id, message.message_type, message.message_id
        );
        Ok(())
    }

    // 接受并处理消息
    pub fn handle_message(
        &mut self,
        message: GossipMessage,
        from_peer: &str,
    ) -> Result<(), String> {
        // 检查是否已经见过这个消息
        if self.seen_messages.contains(&message.message_id) {
            return Ok(());
        }

        self.seen_messages.insert(message.message_id.clone());
        println!(
            "节点 {} 从 {} 接收到消息: {:?} (ID: {})",
            self.node_id, from_peer, message.message_type, message.message_id
        );

        match message.message_type {
            MessageType::Publish => self.handle_publish_message(message, from_peer),
            MessageType::IHave => self.handle_ihave_message(message, from_peer),
            MessageType::IWant => self.handle_iwant_message(message, from_peer),
            MessageType::Graft => self.handle_graft_message(message, from_peer),
            MessageType::Prune => self.handle_prune_message(message, from_peer),
        }
    }

    // 处理发布消息
    fn handle_publish_message(
        &mut self,
        message: GossipMessage,
        from_peer: &str,
    ) -> Result<(), String> {
        if let Some(topic) = &message.topic {
            // 只处理我们订阅的主题
            if !self.topics.contains(topic) {
                return Ok(());
            }

            // 缓存消息
            self.message_cache
                .insert(message.message_id.clone(), message.clone());

            println!(
                "节点 {} 处理发布消息: 主题={}, 内容={:?}",
                self.node_id,
                topic,
                message
                    .content
                    .as_ref()
                    .map(|c| String::from_utf8_lossy(c))
                    .unwrap_or_default()
            );

            // 转发给mesh中的其他节点（除了发送者）
            if let Some(mesh_peers) = self.mesh.get(topic) {
                for peer_id in mesh_peers {
                    if peer_id != from_peer {
                        self.send_message_to_peer(peer_id, &message)?;
                    }
                }
            }
        }
        Ok(())
    }

    // 处理IHAVE消息（暂时简单实现）
    fn handle_ihave_message(
        &mut self,
        _message: GossipMessage,
        _from_peer: &str,
    ) -> Result<(), String> {
        // TODO: 实现IHAVE逻辑
        Ok(())
    }

    // 处理IWANT消息（暂时简单实现）
    fn handle_iwant_message(
        &mut self,
        _message: GossipMessage,
        _from_peer: &str,
    ) -> Result<(), String> {
        // TODO: 实现IWANT逻辑
        Ok(())
    }

    // 处理GRAFT消息（暂时简单实现）
    fn handle_graft_message(
        &mut self,
        _message: GossipMessage,
        _from_peer: &str,
    ) -> Result<(), String> {
        // TODO: 实现GRAFT逻辑
        Ok(())
    }

    // 处理PRUNE消息（暂时简单实现）
    fn handle_prune_message(
        &mut self,
        _message: GossipMessage,
        _from_peer: &str,
    ) -> Result<(), String> {
        // TODO: 实现PRUNE逻辑
        Ok(())
    }

    // 添加对等节点连接
    pub fn add_peer(&mut self, peer_id: String, connection_info: String) {
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
        for (i, peer_id) in available_peers.iter().enumerate() {
            if i >= needed {
                break;
            }
            mesh_peers.insert(peer_id.clone());
        }

        println!(
            "节点 {} 在主题 {} 的mesh中有 {} 个节点",
            self.node_id,
            topic,
            mesh_peers.len()
        );
    }

    // 获取mesh中的节点数量
    pub fn get_mesh_size(&self, topic: &str) -> usize {
        self.mesh.get(topic).map_or(0, |peers| peers.len())
    }

    // 检查是否在某个主题的mesh中
    pub fn is_in_mesh(&self, topic: &str, peer_id: &str) -> bool {
        self.mesh
            .get(topic)
            .map_or(false, |peers| peers.contains(peer_id))
    }
}
