use gossipsub_chat::*;

fn main() {
    println!("=== GossipSub网络 - 第三步测试 ===");

    // 创建几个节点
    let mut node1 = GossipSubNode::new("Node1".to_string());
    let mut node2 = GossipSubNode::new("Node2".to_string());
    let mut node3 = GossipSubNode::new("Node3".to_string());
    let mut node4 = GossipSubNode::new("Node4".to_string());

    // 建立连接 - 创建一个更复杂的网络拓扑
    node1.add_peer("Node2".to_string(), "connection_to_node2".to_string());
    node1.add_peer("Node3".to_string(), "connection_to_node3".to_string());
    node2.add_peer("Node1".to_string(), "connection_to_node1".to_string());
    node2.add_peer("Node4".to_string(), "connection_to_node4".to_string());
    node3.add_peer("Node1".to_string(), "connection_to_node1".to_string());
    node3.add_peer("Node4".to_string(), "connection_to_node4".to_string());
    node4.add_peer("Node2".to_string(), "connection_to_node2".to_string());
    node4.add_peer("Node3".to_string(), "connection_to_node3".to_string());

    // 订阅主题
    node1.subscribe("blockchain".to_string());
    node2.subscribe("blockchain".to_string());
    node3.subscribe("blockchain".to_string());
    node4.subscribe("blockchain".to_string());

    println!("\n=== 测试消息发布 ===");

    // Node1 发布多个消息
    for i in 1..=3 {
        let content = format!("Message {} from Node1", i);
        match node1.publish("blockchain", content.as_bytes().to_vec()) {
            Ok(message_id) => {
                println!("✅ Node1 发布消息 {}: {}", i, message_id);
            }
            Err(e) => {
                println!("❌ Node1 发布消息失败: {}", e);
            }
        }
    }

    println!("\n=== 测试IHAVE/IWANT机制 ===");

    // 模拟gossip心跳
    println!("\n--- Node1 执行gossip心跳 ---");
    match node1.gossip_heartbeat() {
        Ok(_) => println!("✅ Node1 gossip心跳成功"),
        Err(e) => println!("❌ Node1 gossip心跳失败: {}", e),
    }

    // 模拟Node2收到IHAVE消息
    println!("\n--- 模拟Node2收到IHAVE消息 ---");
    let ihave_message = GossipMessage::new(MessageType::IHave)
        .with_topic("blockchain".to_string())
        .with_from("Node1".to_string())
        .with_to("Node2".to_string())
        .with_message_ids(vec!["msg_001".to_string(), "msg_002".to_string()]);

    match node2.handle_message(ihave_message, "Node1") {
        Ok(_) => println!("✅ Node2 成功处理IHAVE消息"),
        Err(e) => println!("❌ Node2 处理IHAVE消息失败: {}", e),
    }

    // 模拟Node1收到IWANT消息
    println!("\n--- 模拟Node1收到IWANT消息 ---");
    let iwant_message = GossipMessage::new(MessageType::IWant)
        .with_topic("blockchain".to_string())
        .with_from("Node2".to_string())
        .with_to("Node1".to_string())
        .with_message_ids(vec!["nonexistent_msg".to_string()]);

    match node1.handle_message(iwant_message, "Node2") {
        Ok(_) => println!("✅ Node1 成功处理IWANT消息"),
        Err(e) => println!("❌ Node1 处理IWANT消息失败: {}", e),
    }

    println!("\n=== 测试缓存清理 ===");
    node1.cleanup_message_cache();
    println!("✅ Node1 执行了缓存清理");

    println!("\n第三步完成！我们新增了：");
    println!("1. gossip_heartbeat() - gossip心跳机制");
    println!("2. send_ihave_messages() - 发送IHAVE消息");
    println!("3. handle_ihave_message() - 处理IHAVE消息");
    println!("4. handle_iwant_message() - 处理IWANT消息");
    println!("5. add_to_gossip_history() - 维护gossip历史");
    println!("6. cleanup_expired_iwant_requests() - 清理过期IWANT请求");
    println!("7. cleanup_message_cache() - 清理过期消息缓存");
    println!("8. 完整的IHAVE/IWANT交互流程");

    println!("\n准备好继续下一步了吗？下一步我们将实现GRAFT/PRUNE mesh管理机制。");
}
