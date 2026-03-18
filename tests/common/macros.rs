#[allow(unused_macros)]
macro_rules! dual_db_test {
    ($name:ident, $body:expr) => {
        paste::paste! {
            #[tokio::test]
            #[serial_test::serial]
            async fn [<$name _sqlite>]() {
                let f: fn(crate::common::db::Backend) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> = |backend| Box::pin(($body)(backend));
                f(crate::common::db::Backend::Sqlite).await;
            }

            #[tokio::test]
            #[serial_test::serial]
            #[ignore]
            async fn [<$name _postgres>]() {
                if std::env::var("TEST_DATABASE_URL").is_err() {
                    eprintln!("TEST_DATABASE_URL not set — skipping Postgres test");
                    return;
                }
                let f: fn(crate::common::db::Backend) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> = |backend| Box::pin(($body)(backend));
                f(crate::common::db::Backend::Postgres).await;
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use dual_db_test;
