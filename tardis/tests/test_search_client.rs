use std::collections::HashMap;
use std::env;

use tardis::basic::result::TardisResult;
use tardis::config::config_dto::{CacheConfig, DBConfig, FrameworkConfig, MQConfig, MailConfig, OSConfig, SearchConfig, SearchModuleConfig, TardisConfig, WebServerConfig};
use tardis::test::test_container::TardisTestContainer;
use tardis::TardisFuns;

#[tokio::test]
async fn test_search_client() -> TardisResult<()> {
    env::set_var("RUST_LOG", "info,tardis=trace");
    TardisFuns::init_log()?;
    TardisTestContainer::es(|url| async move {
        TardisFuns::init_conf(TardisConfig {
            cs: Default::default(),
            fw: FrameworkConfig {
                app: Default::default(),
                web_server: WebServerConfig {
                    enabled: false,
                    ..Default::default()
                },
                web_client: Default::default(),
                cache: CacheConfig {
                    enabled: false,
                    ..Default::default()
                },
                db: DBConfig {
                    enabled: false,
                    ..Default::default()
                },
                mq: MQConfig {
                    enabled: false,
                    ..Default::default()
                },
                search: SearchConfig {
                    enabled: true,
                    url: url.clone(),
                    modules: HashMap::from([(
                        "m1".to_string(),
                        SearchModuleConfig {
                            url: url.clone(),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
                mail: MailConfig {
                    enabled: false,
                    ..Default::default()
                },
                os: OSConfig {
                    enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .await?;

        TardisFuns::search();
        let client = TardisFuns::search_by_module("m1");

        let index_name = "test_index";

        client.create_index(index_name, None).await?;
        assert!(client.check_index_exist(index_name).await?);
        assert!(!client.check_index_exist("test_index_copy").await?);

        client.create_record(index_name, r#"{"user":{"id":1,"name":"张三","open":false}}"#).await?;
        client.create_record(index_name, r#"{"user":{"id":2,"name":"李四","open":false}}"#).await?;
        client.create_record(index_name, r#"{"user":{"id":3,"name":"李四","open":true}}"#).await?;
        let id = client.create_record(index_name, r#"{"user":{"id":4,"name":"Tom","open":true}}"#).await?;

        let record = client.get_record(index_name, &id).await?;
        assert_eq!(record, r#"{"user":{"id":4,"name":"Tom","open":true}}"#);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let records = client.simple_search(index_name, "李四").await?;
        assert_eq!(records.len(), 2);
        assert!(records.contains(&r#"{"user":{"id":2,"name":"李四","open":false}}"#.to_string()));

        let records = client.multi_search(index_name, HashMap::from([("user.id", "1"), ("user.name", "李四")])).await?;
        assert_eq!(records.len(), 0);

        let records = client.multi_search(index_name, HashMap::from([("user.open", "true"), ("user.id", "2"), ("user.name", "李四")])).await?;
        assert_eq!(records.len(), 0);

        let records = client.multi_search(index_name, HashMap::from([("user.open", "false"), ("user.id", "2"), ("user.name", "李四")])).await?;
        assert_eq!(records.len(), 1);
        assert!(records.contains(&r#"{"user":{"id":2,"name":"李四","open":false}}"#.to_string()));

        client
            .delete_by_query(
                index_name,
                r#"{ "query": { "bool": { "must": [{"match": {"user.name": "李四"}}, {"match": {"user.open": "false"}}]}}}"#,
            )
            .await?;
        client.update(index_name, &id, HashMap::from([("user.open".to_string(), "false".to_string()), ("user.xxx".to_string(), "[\"acc01\",\"acc02\"]".to_string())])).await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let raw_search_resp = client.raw_search(index_name, r#"{ "query": { "bool": { "must": [{"match": {"user.name": "李四"}}]}}}"#, Some(10), Some(0)).await?;
        assert_eq!(raw_search_resp.hits.total.value, 1);

        let raw_search_resp = client.raw_search(index_name, r#"{ "query": { "bool": { "must": [{"match": {"user.name": "tom"}}]}}}"#, Some(10), Some(0)).await?;
        assert_eq!(raw_search_resp.hits.hits[0]._source.to_string(), r#"{"user":{"id":4,"name":"Tom","open":"false"}}"#);

        Ok(())
    })
    .await
}
