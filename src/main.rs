use gossipsub_chat::*;

fn main() {
    println!("=== GossipSub网络 - 第四步测试 ===");

    // 创建多个节点来测试mesh管理
    let mut node1 = GossipSubNode::new("Node1".to_string());
    let mut node2 = GossipSubNode::new("Node2".to_string());
    let mut node3 = GossipSubNode::new("Node3".to_string());
    let mut node4 = GossipSubNode::new("Node4".to_string());
    let mut node5 = GossipSubNode::new("Node5".to_string());

    // 建立完全连接的网络
    let nodes = ["Node1", "Node2", "Node3", "Node4", "Node5"];
    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i != j {
                match nodes[i] {
                    "Node1" => {
                        node1.add_peer(nodes[j].to_string(), format!("connection_to_{}", nodes[j]))
                    }
                    "Node2" => {
                        node2.add_peer(nodes[j].to_string(), format!("connection_to_{}", nodes[j]))
                    }
                    "Node3" => {
                        node3.add_peer(nodes[j].to_string(), format!("connection_to_{}", nodes[j]))
                    }
                    "Node4" => {
                        node4.add_peer(nodes[j].to_string(), format!("connection_to_{}", nodes[j]))
                    }
                    "Node5" => {
                        node5.add_peer(nodes[j].to_string(), format!("connection_to_{}", nodes[j]))
                    }
                    _ => {}
                }
            }
        }
    }

    // 所有节点订阅相同主题
    node1.subscribe("blockchain".to_string());
    node2.subscribe("blockchain".to_string());
    node3.subscribe("blockchain".to_string());
    node4.subscribe("blockchain".to_string());
    node5.subscribe("blockchain".to_string());

    println!("\n=== 初始mesh状态 ===");
    println!("Node1 mesh大小: {}", node1.get_mesh_size("blockchain"));
    println!("Node2 mesh大小: {}", node2.get_mesh_size("blockchain"));
    println!("Node3 mesh大小: {}", node3.get_mesh_size("blockchain"));

    println!("\n=== 测试mesh维护 ===");

    // 执行心跳来触发mesh维护
    println!("\n--- Node1 执行心跳维护mesh ---");
    match node1.gossip_heartbeat() {
        Ok(_) => println!("✅ Node1 心跳成功"),
        Err(e) => println!("❌ Node1 心跳失败: {}", e),
    }

    println!("Node1 mesh大小: {}", node1.get_mesh_size("blockchain"));

    println!("\n=== 测试GRAFT/PRUNE消息处理 ===");

    // 模拟Node2向Node1发送GRAFT请求
    println!("\n--- 模拟GRAFT请求 ---");
    let graft_message = GossipMessage::new(MessageType::Graft)
        .with_topic("blockchain".to_string())
        .with_from("Node2".to_string())
        .with_to("Node1".to_string());

    match node1.handle_message(graft_message, "Node2") {
        Ok(_) => println!("✅ Node1 成功处理GRAFT消息"),
        Err(e) => println!("❌ Node1 处理GRAFT消息失败: {}", e),
    }

    // 模拟Node3向Node1发送PRUNE消息
    println!("\n--- 模拟PRUNE消息 ---");
    let prune_message = GossipMessage::new(MessageType::Prune)
        .with_topic("blockchain".to_string())
        .with_from("Node3".to_string())
        .with_to("Node1".to_string());

    match node1.handle_message(prune_message, "Node3") {
        Ok(_) => println!("✅ Node1 成功处理PRUNE消息"),
        Err(e) => println!("❌ Node1 处理PRUNE消息失败: {}", e),
    }

    println!("Node1 最终mesh大小: {}", node1.get_mesh_size("blockchain"));

    println!("\n=== 测试手动mesh扩展和收缩 ===");

    // 手动测试mesh扩展
    println!("\n--- 测试mesh扩展 ---");
    match node2.expand_mesh("blockchain") {
        Ok(_) => println!("✅ Node2 mesh扩展成功"),
        Err(e) => println!("❌ Node2 mesh扩展失败: {}", e),
    }
    println!("Node2 mesh大小: {}", node2.get_mesh_size("blockchain"));

    // 手动测试mesh收缩
    println!("\n--- 测试mesh收缩 ---");
    match node3.contract_mesh("blockchain") {
        Ok(_) => println!("✅ Node3 mesh收缩成功"),
        Err(e) => println!("❌ Node3 mesh收缩失败: {}", e),
    }
    println!("Node3 mesh大小: {}", node3.get_mesh_size("blockchain"));

    println!("\n=== 测试退避机制清理 ===");
    node1.cleanup_backoffs();
    println!("✅ Node1 执行了退避状态清理");

    println!("\n第四步完成！我们新增了：");
    println!("1. maintain_mesh() - mesh网络维护");
    println!("2. expand_mesh() - mesh扩展（发送GRAFT）");
    println!("3. contract_mesh() - mesh收缩（发送PRUNE）");
    println!("4. handle_graft_message() - 处理GRAFT请求");
    println!("5. handle_prune_message() - 处理PRUNE消息");
    println!("6. prune_peer_from_mesh() - 剪除mesh节点");
    println!("7. is_peer_in_backoff() - 退避状态检查");
    println!("8. is_graft_flooding() - GRAFT洪水攻击检测");
    println!("9. cleanup_backoffs() - 清理过期退避状态");
    println!("10. 完整的mesh动态管理机制");

    println!("\n准备好继续下一步了吗？下一步我们将实现评分系统和节点声誉管理。");
}
