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
    pub gossip_history: HashMap<String, Vec<String>>, // topic -> 最近的消息ID列表
    pub iwant_requests: HashMap<String, u64>, // messageId -> 请求时间戳
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
            gossip_history: HashMap::new(),
            iwant_requests: HashMap::new(),
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

        // 添加到gossip历史中
        self.add_to_gossip_history(topic, &message_id);
        Ok(message_id)
    }

    //添加到gossip历史
    fn add_to_gossip_history(&mut self, topic: &str, message_id: &str) {
        let history = self
            .gossip_history
            .entry(topic.to_string())
            .or_insert(Vec::new());
        history.push(message_id.to_string());

        // 保持历史记录在合理大小内
        if history.len() > self.config.gossip_size * 3 {
            history.remove(0);
        }
    }

    // 执行gossip心跳 - 发送IHAVE消息
    pub fn gossip_heartbeat(&mut self) -> Result<(), String> {
        println!("节点 {} 执行gossip心跳", self.node_id);

        for topic in self.topics.clone() {
            self.send_ihave_messages(&topic)?;
        }

        // 清理过期的IWANT请求
        self.cleanup_expired_iwant_requests();

        Ok(())
    }

    fn send_ihave_messages(&mut self, topic: &str) -> Result<(), String> {
        // 获取该主题最近的消息id
        let recent_messages = if let Some(history) = self.gossip_history.get(topic) {
            let start = if history.len() > self.config.gossip_size {
                history.len() - self.config.gossip_size
            } else {
                0
            };
            history[start..].to_vec()
        } else {
            Vec::new()
        };

        if recent_messages.is_empty() {
            return Ok(());
        }

        // 选择要发送的IHAVE消息的节点（非mesh节点）
        let target_peers: Vec<String> = self
            .peers
            .keys()
            .filter(|&peer_id| !self.is_in_mesh(topic, peer_id))
            .take(self.config.gossip_size)
            .cloned()
            .collect();

        for peer_id in &target_peers {
            let ihave_message = GossipMessage::new(MessageType::IHave)
                .with_topic(topic.to_string())
                .with_from(self.node_id.clone())
                .with_to(peer_id.clone())
                .with_message_ids(recent_messages.clone());

            self.send_message_to_peer(&peer_id, &ihave_message)?;
        }

        if !recent_messages.is_empty() {
            println!(
                "节点 {} 向 {} 个节点发送了IHAVE消息，包含 {} 个消息ID",
                self.node_id,
                target_peers.len(),
                recent_messages.len()
            );
        }

        Ok(())
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

    // 处理IHAVE消息
    fn handle_ihave_message(
        &mut self,
        message: GossipMessage,
        from_peer: &str,
    ) -> Result<(), String> {
        if let Some(topic) = &message.topic {
            if !self.topics.contains(topic) {
                return Ok(());
            }

            // 检查我们想要哪些消息
            let mut wanted_messages = Vec::new();
            for message_id in &message.message_ids {
                // 如果我们没有这个消息，且不在我们的缓存中，我们就想要它
                if !self.seen_messages.contains(message_id)
                    && !self.message_cache.contains_key(message_id)
                {
                    wanted_messages.push(message_id.clone());
                }
            }

            if !wanted_messages.is_empty() {
                println!(
                    "节点 {} 从 {} 收到IHAVE消息，想要 {} 个消息",
                    self.node_id,
                    from_peer,
                    wanted_messages.len()
                );

                // 记录IWANT请求时间
                let current_time = GossipMessage::current_timestamp();
                for message_id in &wanted_messages {
                    self.iwant_requests.insert(message_id.clone(), current_time);
                }

                // 发送IWANT消息
                let iwant_message = GossipMessage::new(MessageType::IWant)
                    .with_topic(topic.clone())
                    .with_from(self.node_id.clone())
                    .with_to(from_peer.to_string())
                    .with_message_ids(wanted_messages);

                self.send_message_to_peer(from_peer, &iwant_message)?;
            }
        }
        Ok(())
    }

    // 处理IWANT消息
    fn handle_iwant_message(
        &mut self,
        message: GossipMessage,
        from_peer: &str,
    ) -> Result<(), String> {
        println!(
            "节点 {} 从 {} 收到IWANT消息，请求 {} 个消息",
            self.node_id,
            from_peer,
            message.message_ids.len()
        );

        // 发送请求的消息
        for message_id in &message.message_ids {
            if let Some(cached_message) = self.message_cache.get(message_id) {
                // 创建一个新的消息副本发送给请求者
                let mut response_message = cached_message.clone();
                response_message.to = Some(from_peer.to_string());

                self.send_message_to_peer(from_peer, &response_message)?;
                println!("  发送消息 {} 给 {}", message_id, from_peer);
            } else {
                println!("  消息 {} 不在缓存中，无法发送", message_id);
            }
        }
        Ok(())
    }

    // 清理过期的IWANT请求
    fn cleanup_expired_iwant_requests(&mut self) {
        let current_time = GossipMessage::current_timestamp();
        let ttl = self.config.message_cache_ttl;

        self.iwant_requests
            .retain(|_, &mut timestamp| current_time - timestamp < ttl);
    }

    // 清理过期的消息缓存
    pub fn cleanup_message_cache(&mut self) {
        let current_time = GossipMessage::current_timestamp();
        let ttl = self.config.message_cache_ttl;

        self.message_cache
            .retain(|_, message| current_time - message.timestamp < ttl);

        // 同时清理seen_messages中的过期项
        // 注意：这里简化处理，实际应该记录消息的时间戳
        if self.seen_messages.len() > 1000 {
            self.seen_messages.clear();
        }
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
