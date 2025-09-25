// 消息类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    IHave,
    IWant,
    Graft, // 请求加入mesh
    Prune, // 请求离开mesh
    Publish,
}

// GossipSub配置
#[derive(Debug, Clone)]
pub struct GossipSubConfig {
    pub mesh_size: usize,           // 每个topic的mesh大小
    pub mesh_low: usize,            // mesh最小大小
    pub mesh_high: usize,           // mesh最大大小
    pub gossip_size: usize,         // gossip消息数量
    pub heartbeat_interval: u64,    // 心跳间隔(ms)
    pub message_cache_ttl: u64,     // 消息缓存时间(ms)
    pub graft_flood_threshold: u64, // GRAFT洪水攻击阈值(ms)
    pub prune_backoff: u64,         // PRUNE后的退避时间(ms)
    pub graft_backoff: u64,         // GRAFT被拒绝后的退避时间(ms)
}

impl Default for GossipSubConfig {
    fn default() -> Self {
        Self {
            mesh_size: 6,
            mesh_low: 4,
            mesh_high: 12,
            gossip_size: 3,
            heartbeat_interval: 1000,
            message_cache_ttl: 30000,
            graft_flood_threshold: 10000, // 10秒
            prune_backoff: 60000,         // 1分钟
            graft_backoff: 60000,         // 1分钟
        }
    }
}
