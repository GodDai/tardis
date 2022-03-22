use std::collections::HashMap;

use crate::basic::error::TardisError;
use crate::basic::result::TardisResult;
use crate::log::{debug, info};
use crate::{FrameworkConfig, TardisFuns, TardisWebClient};

/// Distributed search handle / 分布式搜索操作
///
/// Encapsulates common elasticsearch operations.
///
/// 封装了Elasticsearch的常用操作.
///
/// # Steps to use / 使用步骤
///
/// 1. Create the search configuration / 创建搜索配置, @see [SearchConfig](crate::basic::config::SearchConfig)
///
/// 2. Use `TardisSearchClient` to operate search / 使用 `TardisSearchClient` 操作搜索, E.g:
/// ```rust
/// use tardis::TardisFuns;
/// TardisFuns::search().create_index("test_index").await.unwrap();
/// let id = TardisFuns::search().create_record("test_index", r#"{"user":{"id":1,"name":"张三","open":false}}"#).await.unwrap();
/// assert_eq!(TardisFuns::search().get_record("test_index", &id).await.unwrap(), r#"{"user":{"id":4,"name":"Tom","open":true}}"#);
/// TardisFuns::search().simple_search("test_index", "张三").await.unwrap();
/// ```
pub struct TardisSearchClient {
    client: TardisWebClient,
    server_url: String,
}

impl TardisSearchClient {
    /// Initialize configuration from the search configuration object / 从搜索配置对象中初始化配置
    pub fn init_by_conf(conf: &FrameworkConfig) -> TardisResult<TardisSearchClient> {
        TardisSearchClient::init(&conf.search.url, conf.search.timeout_sec)
    }

    /// Initialize configuration / 初始化配置
    pub fn init(str_url: &str, timeout_sec: u64) -> TardisResult<TardisSearchClient> {
        info!("[Tardis.SearchClient] Initializing");
        let mut client = TardisWebClient::init(timeout_sec)?;
        client.set_default_header("Content-Type", "application/json");
        info!("[Tardis.SearchClient] Initialized");
        TardisResult::Ok(TardisSearchClient {
            client,
            server_url: str_url.to_string(),
        })
    }

    /// Create index / 创建索引
    ///
    /// # Arguments
    ///
    ///  * `index_name` -  index name / 索引名称
    ///
    /// # Examples
    /// ```rust
    /// use tardis::TardisFuns;
    /// TardisFuns::search().create_index("test_index").await.unwrap();
    /// ```
    pub async fn create_index(&self, index_name: &str) -> TardisResult<()> {
        info!("[Tardis.SearchClient] Create index {}", index_name);
        let url = format!("{}/{}", self.server_url, index_name);
        let resp = self.client.put_str_to_str(&url, "", None).await?;
        if let Some(err) = TardisError::new(resp.code, resp.body.as_ref().unwrap_or(&"".to_string())) {
            Err(err)
        } else {
            Ok(())
        }
    }

    /// Create record and return primary key value  / 创建记录并返回主键值
    ///
    /// # Arguments
    ///
    ///  * `index_name` -  index name / 索引名称
    ///  * `data` -  record content / 记录内容
    ///
    /// # Examples
    /// ```rust
    /// use tardis::TardisFuns;
    /// let id = TardisFuns::search().create_record("test_index", r#"{"user":{"id":1,"name":"张三","open":false}}"#).await.unwrap();
    /// ```
    pub async fn create_record(&self, index_name: &str, data: &str) -> TardisResult<String> {
        debug!("[Tardis.SearchClient] Create index {}", index_name);
        let url = format!("{}/{}/_doc/", self.server_url, index_name);
        let resp = self.client.post_str_to_str(&url, data, None).await?;
        if let Some(err) = TardisError::new(resp.code, resp.body.as_ref().unwrap_or(&"".to_string())) {
            Err(err)
        } else {
            let result = TardisFuns::json.str_to_json(&resp.body.unwrap_or_else(|| "".to_string()))?;
            Ok(result["_id"].as_str().ok_or_else(|| TardisError::FormatError("[Tardis.SearchClient] [_id] structure not found".to_string()))?.to_string())
        }
    }

    /// Get a record  / 获取一条记录
    ///
    /// # Arguments
    ///
    ///  * `index_name` -  index name / 索引名称
    ///  * `id` -  record primary key value / 记录主键值
    ///
    /// # Examples
    /// ```rust
    /// use tardis::TardisFuns;
    /// TardisFuns::search().get_record("test_index", "xxxx").await.unwrap();
    /// ```
    pub async fn get_record(&self, index_name: &str, id: &str) -> TardisResult<String> {
        let url = format!("{}/{}/_doc/{}", self.server_url, index_name, id);
        let resp = self.client.get_to_str(&url, None).await?;
        if let Some(err) = TardisError::new(resp.code, resp.body.as_ref().unwrap_or(&"".to_string())) {
            Err(err)
        } else {
            let result = TardisFuns::json.str_to_json(&resp.body.unwrap_or_else(|| "".to_string()))?;
            Ok(result["_source"].to_string())
        }
    }

    /// Simple (global) search  / 简单（全局）搜索
    ///
    /// # Arguments
    ///
    ///  * `index_name` -  index name / 索引名称
    ///  * `q` -  keyword / 搜索关键字
    ///
    /// # Examples
    /// ```rust
    /// use tardis::TardisFuns;
    /// TardisFuns::search().simple_search("test_index", "张三").await.unwrap();
    /// ```
    pub async fn simple_search(&self, index_name: &str, q: &str) -> TardisResult<Vec<String>> {
        let url = format!("{}/{}/_search?q={}", self.server_url, index_name, q);
        let resp = self.client.get_to_str(&url, None).await?;
        if let Some(err) = TardisError::new(resp.code, resp.body.as_ref().unwrap_or(&"".to_string())) {
            Err(err)
        } else {
            Self::parse_search_result(&resp.body.unwrap_or_else(|| "".to_string()))
        }
    }

    /// Specified fields search  / 指定字段搜索
    ///
    /// # Arguments
    ///
    ///  * `index_name` -  index name / 索引名称
    ///  * `q` -  search fields / 搜索的字段集合
    ///
    /// The format of the search field: key = field name , value = field value, exact match, key supports multi-level operations of Json.
    ///
    /// 搜索字段的格式: key = 字段名 , value = 字段值，精确匹配，key支持Json的多级操作.
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use tardis::TardisFuns;
    /// TardisFuns::search().multi_search(index_name, HashMap::from([("user.id", "1"), ("user.name", "李四")])).await.unwrap();
    /// ```
    pub async fn multi_search(&self, index_name: &str, q: HashMap<&str, &str>) -> TardisResult<Vec<String>> {
        let q = q.into_iter().map(|(k, v)| format!(r#"{{"match": {{"{}": "{}"}}}}"#, k, v)).collect::<Vec<String>>().join(",");
        let q = format!(r#"{{ "query": {{ "bool": {{ "must": [{}]}}}}}}"#, q);
        self.raw_search(index_name, &q).await
    }

    /// Search using native format  / 使用原生格式搜索
    ///
    /// # Arguments
    ///
    ///  * `index_name` -  index name / 索引名称
    ///  * `q` -  native format / 原生格式
    ///
    pub async fn raw_search(&self, index_name: &str, q: &str) -> TardisResult<Vec<String>> {
        let url = format!("{}/{}/_search", self.server_url, index_name);
        let resp = self.client.post_str_to_str(&url, q, None).await?;
        if let Some(err) = TardisError::new(resp.code, resp.body.as_ref().unwrap_or(&"".to_string())) {
            Err(err)
        } else {
            Self::parse_search_result(&resp.body.unwrap_or_else(|| "".to_string()))
        }
    }

    fn parse_search_result(result: &str) -> TardisResult<Vec<String>> {
        let json = TardisFuns::json.str_to_json(result)?;
        let json = json["hits"]["hits"]
            .as_array()
            .ok_or_else(|| TardisError::FormatError("[Tardis.SearchClient] [hit.hit] structure not found".to_string()))?
            .iter()
            .map(|x| x["_source"].to_string())
            .collect();
        Ok(json)
    }
}
