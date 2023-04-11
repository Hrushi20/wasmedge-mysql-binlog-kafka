use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::providers::mariadb::gtid::gtid_list::GtidList;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::events::event_header::EventHeader;

use rskafka::{
    client::{
        ClientBuilder,
        partition::{Compression, UnknownTopicHandling},
    },
    record::Record,
};
use chrono::{ TimeZone, Utc };

async fn send_events_to_kafka(event_header:&EventHeader, event:&BinlogEvent){
    let connection = "localhost:9092".to_owned();
    let client = ClientBuilder::new(vec![connection]).build().await.expect("Couldn't connect to kafka");
    let topic = "sql_binlog_events";
    let controller_client = client.controller_client().unwrap();

    let event_binary = serde_json::to_vec(event).unwrap();
    // controller_client.create_topic(
    //     topic,
    //     2,      // partitions
    //     1,      // replication factor
    //     5_000,  // timeout (ms)
    // ).await.unwrap();
    //
    // // get a partition-bound client
    // let partition_client = client
    //     .partition_client(
    //         topic.to_owned(),
    //         0,  // partition
    //         UnknownTopicHandling::Retry,
    //     )
    //     .await
    //     .unwrap();
    //
    // let
    // // produce some data
    // let record = Record {
    //     key: None,
    //     value: Some(event.),
    //     headers: BTreeMap::from([
    //         ("foo".to_owned(), b"bar".to_vec()),
    //     ]),
    //     timestamp: Utc.timestamp_millis(42),
    // };
    // partition_client.produce(vec![record], Compression::default()).await.unwrap();

}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(),mysql_cdc::errors::Error> {
    // Start replication from MariaDB GTID
    let _options = BinlogOptions::from_mariadb_gtid(GtidList::parse("0-1-270")?);

    // Start replication from MySQL GTID
    let gtid_set =
        "d4c17f0c-4f11-11ea-93e3-325d3e1cd1c8:1-107, f442510a-2881-11ea-b1dd-27916133dbb2:1-7";
    let _options = BinlogOptions::from_mysql_gtid(GtidSet::parse(gtid_set)?);

    // Start replication from the position
    let _options = BinlogOptions::from_position(String::from("mysql-bin.000008"), 195);

    // Start replication from last master position.
    // Useful when you are only interested in new changes.
    let _options = BinlogOptions::from_end();

    // Start replication from first event of first available master binlog.
    // Note that binlog files by default have expiration time and deleted.
    let options = BinlogOptions::from_start();

    let options = ReplicaOptions {
        username: String::from("root"),
        password: String::from("Hrushi20"),
        blocking: true,
        ssl_mode: SslMode::Disabled,
        binlog: options,
        ..Default::default()
    };

    let mut client = BinlogClient::new(options);

    for result in client.replicate()? {
        let (header, event) = result?;
        println!("Header: {:#?}", header);
        // println!("Event: {:#?}", event);
        send_events_to_kafka(&header,&event).await;

        // After you processed the event, you need to update replication position
        client.commit(&header, &event);
    }
    Ok(())
}