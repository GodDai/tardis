use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

/// Message queue configuration / 消息队列配置
///
/// Message queue operation needs to be enabled ```#[cfg(feature = "mq")]``` .
///
/// 消息队列操作需要启用 ```#[cfg(feature = "mq")]``` .
///
/// # Examples
/// ```ignore
/// use tardis::basic::config::MQModuleConfig;
/// let config = MQModuleConfig {
///    url: "amqp://guest:guest@127.0.0.1:5672/%2f".parse().unwrap(),
///    ..Default::default()
///};
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TypedBuilder)]
pub struct MQModuleConfig {
    /// Message queue access Url, Url with permission information / 消息队列访问Url，Url带权限信息
    pub url: Url,
}
