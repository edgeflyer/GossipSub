use gossipsub_chat::*;

fn main() {
    println!("=== GossipSub网络 - 第一步测试 ===");

    // 创建几个节点
    let mut node1 = GossipSubNode::new("Node1".to_string());
    let mut node2 = GossipSubNode::new("Node2".to_string());
    let mut node3 = GossipSubNode::new("Node3".to_string());

    // 建立连接 (这里用简单的字符串表示连接信息)
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

    // 创建一个示例消息
    let message = GossipMessage::new(MessageType::Publish)
        .with_topic("blockchain".to_string())
        .with_content(b"Hello GossipSub!".to_vec())
        .with_from("Node1".to_string());

    println!("\n创建的示例消息:");
    println!("  ID: {}", message.message_id);
    println!("  类型: {:?}", message.message_type);
    println!("  主题: {:?}", message.topic);
    println!("  内容: {:?}", String::from_utf8_lossy(message.content.as_ref().unwrap()));

    println!("\n第一步完成！我们创建了：");
    println!("1. MessageType枚举 - 定义消息类型");
    println!("2. GossipMessage结构 - 用于封装各种消息");
    println!("3. GossipSubConfig结构 - 配置参数");
    println!("4. GossipSubNode结构 - 基本的节点结构");
    println!("5. 节点连接和主题订阅功能");
    println!("6. 基础的mesh网络初始化");

    println!("\n准备好继续下一步了吗？下一步我们将实现消息发布和转发机制。");
}
