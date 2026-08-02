use typedb_driver::{Addresses, Credentials, DriverOptions, DriverTlsConfig, TypeDBDriver};

#[tokio::test]
#[ignore = "requires a live TypeDB instance: docker compose -f docker-compose.calvin.yml up -d typedb"]
async fn connects_to_local_typedb() {
    let credentials = Credentials::new("admin", "password");
    let tls_config = DriverTlsConfig::disabled();
    let options = DriverOptions::new(tls_config);
    let addresses = Addresses::try_from_address_str("localhost:1729").expect("parse address");

    let driver = TypeDBDriver::new(addresses, credentials, options)
        .await
        .expect("connect to local TypeDB");

    let dbs = driver.databases();
    let db_name = "harkonnen_connectivity_check";
    if !dbs.contains(db_name).await.expect("check database exists") {
        dbs.create(db_name).await.expect("create test database");
    }
    assert!(dbs
        .contains(db_name)
        .await
        .expect("verify database created"));
}
