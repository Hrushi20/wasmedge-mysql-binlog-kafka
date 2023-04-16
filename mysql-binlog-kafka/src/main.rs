use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::providers::mariadb::gtid::gtid_list::GtidList;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::events::event_header::EventHeader;

use std::collections::BTreeMap;
use rskafka::{
    client::{
        ClientBuilder,
        partition::{Compression, UnknownTopicHandling},
    },
    record::Record,
};
use chrono::{ TimeZone, Utc };
use rskafka::client::Client;
use rskafka::client::partition::{OffsetAt, PartitionClient};

struct KafkaProducer {
    client: Client,
    topic: Option<String>
}

impl KafkaProducer {

    async fn connect(url: String) -> Self {
       KafkaProducer {
           client: ClientBuilder::new(vec![url]).build().await.expect("Couldn't connect to kafka"),
           topic: None
       }
    }

    async fn create_topic(&mut self, topic_name: &str){
        let topics = self.client.list_topics().await.unwrap();

        for topic in topics {
            if topic.name.eq(&topic_name.to_string()) {
                self.topic = Some(topic_name.to_string());
                println!("Topic already exist in Kafka");
                return
            }
        }

        let controller_client = self.client.controller_client().expect("Couldn't create controller client kafka");
        controller_client.create_topic(
            topic_name,
            1,      // partitions
            1,      // replication factor
            5_000,  // timeout (ms)
        ).await.unwrap();
        self.topic = Some(topic_name.to_string());
    }

    fn create_record(&self,headers:String,value:String) -> Record{
        Record {
            key: None,
            value: Some(value.into_bytes()),
            headers: BTreeMap::from([
                ("mysql_binlog_headers".to_owned(), headers.into_bytes()),
            ]),
            timestamp: Utc.timestamp_millis(42),
        }
    }

    async fn get_partition_client(&self,partition:i32) -> Option<PartitionClient>{
        if self.topic.is_none() {
            ()
        }

        let topic = self.topic.as_ref().unwrap();
        Some(self.client.partition_client(topic,partition,UnknownTopicHandling::Retry).await.expect("Couldn't fetch controller client"))
    }

}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(),mysql_cdc::errors::Error> {
    // // Start replication from MariaDB GTID
    // let _options = BinlogOptions::from_mariadb_gtid(GtidList::parse("0-1-270")?);
    //
    // // Start replication from MySQL GTID
    // let gtid_set =
    //     "d4c17f0c-4f11-11ea-93e3-325d3e1cd1c8:1-107, f442510a-2881-11ea-b1dd-27916133dbb2:1-7";
    // let _options = BinlogOptions::from_mysql_gtid(GtidSet::parse(gtid_set)?);
    //
    // // Start replication from the position
    // let _options = BinlogOptions::from_position(String::from("mysql-bin.000008"), 195);
    //
    // Start replication from last master position.
    // Useful when you are only interested in new changes.
    let options = BinlogOptions::from_end();

    // Start replication from first event of first available master binlog.
    // Note that binlog files by default have expiration time and deleted.
    // let options = BinlogOptions::from_start();

    let options = ReplicaOptions {
        username: String::from("root"),
        password: String::from("password"),
        blocking: true,
        ssl_mode: SslMode::Disabled,
        binlog: options,
        ..Default::default()
    };

    let mut client = BinlogClient::new(options);

    let mut kafka_producer = KafkaProducer::connect("localhost:9092".to_string()).await;
    kafka_producer.create_topic("mysql_binlog_events").await;
    let partitionClient = kafka_producer.get_partition_client(0).await.unwrap();
    let mut partition_offset = partitionClient.get_offset(OffsetAt::Latest).await.unwrap();

    for result in client.replicate()? {
        let (header, event) = result?;

        let json_event = serde_json::to_string(&event).expect("Couldn't convert sql event to json");
        let json_header = serde_json::to_string(&header).expect("Couldn't convert sql header to json");

        let kafka_record = kafka_producer.create_record(json_header,json_event);
        partitionClient.produce(vec![kafka_record],Compression::default()).await.unwrap();


        // Consumer
        let (records, high_watermark) = partitionClient
            .fetch_records(
                partition_offset,  // offset
                1..100_000,  // min..max bytes
                1_000,  // max wait time
            )
            .await
            .unwrap();

        partition_offset = high_watermark;

        for record in records {
            let record_clone = record.clone();
            let timestamp = record_clone.record.timestamp;
            let value = record_clone.record.value.unwrap();
            let header = record_clone.record.headers.get("mysql_binlog_headers").unwrap().clone();

            println!("============================================== Event from Apache kafka ==========================================================================");
            println!();
            println!("Value: {}",String::from_utf8(value).unwrap());
            println!("Timestamp: {}",timestamp);
            println!("Headers: {}",String::from_utf8(header).unwrap());
            println!();
            println!();

        }

        // After you processed the event, you need to update replication position
        client.commit(&header, &event);
    }
    Ok(())
}