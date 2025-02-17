use std::collections::HashMap;

use deadpool_redis::{Config, Connection, Pool, Runtime};
use redis::{AsyncCommands, ErrorKind, RedisError, RedisResult};
use tracing::{error, info, trace};

use crate::basic::error::TardisError;
use crate::basic::result::TardisResult;
use crate::config::config_dto::component::cache::CacheModuleConfig;

use crate::utils::initializer::InitBy;

/// Distributed cache handle / 分布式缓存操作
///
/// Encapsulates common Redis operations.
///
/// 封装了Redis的常用操作.
///
/// # Steps to use / 使用步骤
///
/// 1. Create the cache configuration / 创建缓存配置, @see [CacheConfig](crate::basic::config::CacheConfig)
///
/// 4. Use `TardisCacheClient` to operate cache / 使用 `TardisCacheClient` 操作缓存, E.g:
/// ```ignore
/// use tardis::TardisFuns;
/// assert_eq!(TardisFuns::cache().get("test_key").await.unwrap(), None);
/// client.set("test_key", "测试").await.unwrap();
/// assert_eq!(TardisFuns::cache().get("test_key").await.unwrap(), "测试");
/// assert!(TardisFuns::cache().set_nx("test_key2", "测试2").await.unwrap());
/// assert!(!TardisFuns::cache().set_nx("test_key2", "测试2").await.unwrap());
/// ```
pub struct TardisCacheClient {
    pool: Pool,
}
#[async_trait::async_trait]
impl InitBy<CacheModuleConfig> for TardisCacheClient {
    async fn init_by(config: &CacheModuleConfig) -> TardisResult<Self> {
        Self::init(config).await
    }
}

impl TardisCacheClient {
    /// Initialize configuration / 初始化配置
    pub async fn init(CacheModuleConfig { url }: &CacheModuleConfig) -> TardisResult<TardisCacheClient> {
        info!(
            "[Tardis.CacheClient] Initializing, host:{}, port:{}, db:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            if url.path().is_empty() { "" } else { &url.path()[1..] },
        );
        let cfg = Config::from_url(url.clone());
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| TardisError::format_error(&format!("[Tardis.CacheClient] Create pool error: {e}"), "500-tardis-cache-pool-error"))?;
        info!(
            "[Tardis.CacheClient] Initialized, host:{}, port:{}, db:{}",
            url.host_str().unwrap_or(""),
            url.port().unwrap_or(0),
            if url.path().is_empty() { "" } else { &url.path()[1..] },
        );
        Ok(TardisCacheClient { pool })
    }

    async fn get_connection(&self) -> RedisResult<Connection> {
        self.pool.get().await.map_err(|error| RedisError::from((ErrorKind::IoError, "Get connection error", error.to_string())))
    }

    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] set, key:{}, value:{}", key, value);
        self.get_connection().await?.set(key, value).await
    }

    pub async fn set_ex(&self, key: &str, value: &str, ex_sec: usize) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] set_ex, key:{}, value:{}, ex_sec:{}", key, value, ex_sec);
        self.get_connection().await?.set_ex(key, value, ex_sec).await
    }

    pub async fn set_nx(&self, key: &str, value: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] set_nx, key:{}, value:{}", key, value);
        self.get_connection().await?.set_nx(key, value).await
    }

    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        trace!("[Tardis.CacheClient] get, key:{}", key);
        self.get_connection().await?.get(key).await
    }

    pub async fn getset(&self, key: &str, value: &str) -> RedisResult<Option<String>> {
        trace!("[Tardis.CacheClient] getset, key:{}, value:{}", key, value);
        self.get_connection().await?.getset(key, value).await
    }

    pub async fn incr(&self, key: &str, delta: isize) -> RedisResult<isize> {
        trace!("[Tardis.CacheClient] incr, key:{}, delta:{}", key, delta);
        self.get_connection().await?.incr(key, delta).await
    }

    pub async fn del(&self, key: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] del, key:{}", key);
        self.get_connection().await?.del(key).await
    }

    pub async fn del_confirm(&self, key: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] del_confirm, key:{}", key);
        self.del(key).await?;
        loop {
            match self.exists(key).await {
                Ok(false) => {
                    return Ok(());
                }
                Err(error) => {
                    return Err(error);
                }
                _ => {}
            }
        }
    }

    pub async fn exists(&self, key: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] exists, key:{}", key);
        self.get_connection().await?.exists(key).await
    }

    pub async fn expire(&self, key: &str, ex_sec: usize) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] expire, key:{}, ex_sec:{}", key, ex_sec);
        self.get_connection().await?.expire(key, ex_sec).await
    }

    pub async fn expire_at(&self, key: &str, timestamp_sec: usize) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] expire_at, key:{}, timestamp_sec:{}", key, timestamp_sec);
        self.get_connection().await?.expire_at(key, timestamp_sec).await
    }

    pub async fn ttl(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] ttl, key:{}", key);
        self.get_connection().await?.ttl(key).await
    }

    // list operations

    pub async fn lpush(&self, key: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] lpush, key:{}, value:{}", key, value);
        self.get_connection().await?.lpush(key, value).await
    }

    pub async fn rpush(&self, key: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] rpush, key:{}, value:{}", key, value);
        self.get_connection().await?.rpush(key, value).await
    }

    pub async fn lrangeall(&self, key: &str) -> RedisResult<Vec<String>> {
        trace!("[Tardis.CacheClient] lrangeall, key:{}", key);
        self.get_connection().await?.lrange(key, 0, -1).await
    }

    pub async fn llen(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] llen, key:{}", key);
        self.get_connection().await?.llen(key).await
    }

    pub async fn lrem(&self, key: &str, count: isize, value: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] lrem, key:{}", key);
        self.get_connection().await?.lrem(key, count, value).await
    }

    pub async fn linsert_after(&self, key: &str, count: isize, value: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] linsert_after, key:{}", key);
        self.get_connection().await?.linsert_after(key, count, value).await
    }

    pub async fn linsert_before(&self, key: &str, count: isize, value: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] linsert_before, key:{}", key);
        self.get_connection().await?.linsert_before(key, count, value).await
    }

    pub async fn lset(&self, key: &str, count: isize, value: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] lset, key:{}", key);
        self.get_connection().await?.lset(key, count, value).await
    }

    // hash operations

    pub async fn hget(&self, key: &str, field: &str) -> RedisResult<Option<String>> {
        trace!("[Tardis.CacheClient] hget, key:{}, field:{}", key, field);
        self.get_connection().await?.hget(key, field).await
    }

    pub async fn hset(&self, key: &str, field: &str, value: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] hset, key:{}, field:{}, value:{}", key, field, value);
        self.get_connection().await?.hset(key, field, value).await
    }

    pub async fn hset_nx(&self, key: &str, field: &str, value: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] hset_nx, key:{}, field:{}, value:{}", key, field, value);
        self.get_connection().await?.hset_nx(key, field, value).await
    }

    pub async fn hdel(&self, key: &str, field: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] hdel, key:{}, field:{}", key, field);
        self.get_connection().await?.hdel(key, field).await
    }

    pub async fn hdel_confirm(&self, key: &str, field: &str) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] hdel_confirm, key:{}, field:{}", key, field);
        self.hdel(key, field).await?;
        loop {
            match self.hexists(key, field).await {
                Ok(false) => {
                    return Ok(());
                }
                Err(error) => {
                    return Err(error);
                }
                _ => {}
            }
        }
    }

    pub async fn hincr(&self, key: &str, field: &str, delta: isize) -> RedisResult<isize> {
        trace!("[Tardis.CacheClient] hincr, key:{}, field:{}, delta:{}", key, field, delta);
        self.get_connection().await?.hincr(key, field, delta).await
    }

    pub async fn hexists(&self, key: &str, field: &str) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] hexists, key:{}, field:{}", key, field);
        self.get_connection().await?.hexists(key, field).await
    }

    pub async fn hkeys(&self, key: &str) -> RedisResult<Vec<String>> {
        trace!("[Tardis.CacheClient] hkeys, key:{}", key);
        self.get_connection().await?.hkeys(key).await
    }

    pub async fn hvals(&self, key: &str) -> RedisResult<Vec<String>> {
        trace!("[Tardis.CacheClient] hvals, key:{}", key);
        self.get_connection().await?.hvals(key).await
    }

    pub async fn hgetall(&self, key: &str) -> RedisResult<HashMap<String, String>> {
        trace!("[Tardis.CacheClient] hgetall, key:{}", key);
        self.get_connection().await?.hgetall(key).await
    }

    pub async fn hlen(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] hlen, key:{}", key);
        self.get_connection().await?.hlen(key).await
    }

    // bitmap operations

    pub async fn setbit(&self, key: &str, offset: usize, value: bool) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] setbit, key:{}, offset:{}, value:{}", key, offset, value);
        self.get_connection().await?.setbit(key, offset, value).await
    }

    pub async fn getbit(&self, key: &str, offset: usize) -> RedisResult<bool> {
        trace!("[Tardis.CacheClient] getbit, key:{}, offset:{}", key, offset);
        self.get_connection().await?.getbit(key, offset).await
    }

    pub async fn bitcount(&self, key: &str) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] bitcount, key:{}", key);
        self.get_connection().await?.bitcount(key).await
    }

    pub async fn bitcount_range_by_byte(&self, key: &str, start: usize, end: usize) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] bitcount_range_by_byte, key:{}, start:{}, end:{}", key, start, end);
        self.get_connection().await?.bitcount_range(key, start, end).await
    }

    /// Supported from version redis 7.0.0
    pub async fn bitcount_range_by_bit(&self, key: &str, start: usize, end: usize) -> RedisResult<usize> {
        trace!("[Tardis.CacheClient] bitcount_range_by_bit, key:{}, start:{}, end:{}", key, start, end);
        match redis::cmd("BITCOUNT").arg(key).arg(start).arg(end).arg("BIT").query_async(&mut self.get_connection().await?).await {
            Ok(count) => Ok(count),
            Err(error) => Err(error),
        }
    }

    // other operations

    pub async fn flushdb(&self) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] flushdb");
        match redis::cmd("FLUSHDB").query_async(&mut self.get_connection().await?).await {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub async fn flushall(&self) -> RedisResult<()> {
        trace!("[Tardis.CacheClient] flushall");
        match redis::cmd("FLUSHALL").query_async(&mut self.get_connection().await?).await {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        }
    }

    // custom
    pub async fn cmd(&self) -> RedisResult<Connection> {
        self.get_connection().await
    }
}

impl From<RedisError> for TardisError {
    fn from(error: RedisError) -> Self {
        error!("[Tardis.CacheClient] [{}]{}", error.code().unwrap_or(""), error.detail().unwrap_or(""));
        TardisError::wrap(&format!("[Tardis.CacheClient] {error:?}"), "-1-tardis-cache-error")
    }
}
