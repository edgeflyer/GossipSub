1. MessageType 枚举 - 定义消息类型
2. GossipMessage 结构 - 用于封装各种消息
3. GossipSubConfig 结构 - 配置参数
4. GossipSubNode 结构 - 基本的节点结构
5. 节点连接和主题订阅功能
6. 基础的 mesh 网络初始化

7. gossip_heartbeat() - gossip 心跳机制
8. send_ihave_messages() - 发送 IHAVE 消息
9. handle_ihave_message() - 处理 IHAVE 消息
10. handle_iwant_message() - 处理 IWANT 消息
11. add_to_gossip_history() - 维护 gossip 历史
12. cleanup_expired_iwant_requests() - 清理过期 IWANT 请求
13. cleanup_message_cache() - 清理过期消息缓存
14. 完整的 IHAVE/IWANT 交互流程
