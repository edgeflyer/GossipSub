use gossipsub_chat::*;

fn main() {
    println!("=== GossipSub网络 - 第二步测试 ===");

    // 创建几个节点
    let mut node1 = GossipSubNode::new("Node1".to_string());
    let mut node2 = GossipSubNode::new("Node2".to_string());
    let mut node3 = GossipSubNode::new("Node3".to_string());

    // 建立连接
    node1.add_peer("Node2".to_string(), "connection_to_node2".to_string());
    node1.add_peer("Node3".to_string(), "connection_to_node3".to_string());
    node2.add_peer("Node1".to_string(), "connection_to_node1".to_string());
    node2.add_peer("Node3".to_string(), "connection_to_node3".to_string());
    node3.add_peer("Node1".to_string(), "connection_to_node1".to_string());
    node3.add_peer("Node2".to_string(), "connection_to_node2".to_string());

    // 订阅主题
    node1.subscribe("blockchain".to_string());
    node2.subscribe("blockchain".to_string());
    node3.subscribe("blockchain".to_string());

    println!("\n=== 测试消息发布和转发 ===");

    // Node1 发布消息
    match node1.publish("blockchain", b"Hello from Node1!".to_vec()) {
        Ok(message_id) => {
            println!("✅ Node1 成功发布消息，ID: {}", message_id);
        }
        Err(e) => {
            println!("❌ Node1 发布消息失败: {}", e);
        }
    }

    println!("\n=== 模拟消息接收和转发 ===");

    // 模拟Node2接收到来自Node1的消息
    let test_message = GossipMessage::new(MessageType::Publish)
        .with_topic("blockchain".to_string())
        .with_content(b"Message from external source".to_vec())
        .with_from("External".to_string());

    match node2.handle_message(test_message, "Node1") {
        Ok(_) => println!("✅ Node2 成功处理接收到的消息"),
        Err(e) => println!("❌ Node2 处理消息失败: {}", e),
    }

    // 测试向未订阅主题发布消息
    println!("\n=== 测试错误情况 ===");
    match node1.publish("unknown_topic", b"This should fail".to_vec()) {
        Ok(_) => println!("❌ 不应该成功"),
        Err(e) => println!("✅ 预期的错误: {}", e),
    }

    println!("\n第二步完成！我们新增了：");
    println!("1. publish() - 消息发布功能");
    println!("2. forward_to_mesh() - mesh网络转发");
    println!("3. forward_to_fanout() - fanout网络转发");
    println!("4. handle_message() - 消息处理调度");
    println!("5. handle_publish_message() - 处理发布消息");
    println!("6. send_message_to_peer() - 发送消息给对等节点");
    println!("7. 消息去重机制 (seen_messages)");
    println!("8. 消息缓存机制 (message_cache)");

    println!("\n准备好继续下一步了吗？下一步我们将实现IHAVE/IWANT gossip机制。");
}
