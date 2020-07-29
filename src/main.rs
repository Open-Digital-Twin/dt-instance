use tokio::{task,time};

use rumqttc::{MqttOptions, QoS, EventLoop, Request, Subscribe, Publish, Incoming, Outgoing};
use async_channel::{Sender};

use chrono::prelude::*;

use std::time::Duration;
use std::env;

extern crate env_logger;

use log::{info, error};

#[macro_use]
#[allow(unused_imports)]
extern crate cdrs;

#[macro_use]
extern crate cdrs_helpers_derive;
use cdrs::query::*;
use crate::cdrs::frame::TryFromRow; 

mod common;
use common::db::get_db_session;
use common::models::twin::*;

use uuid::Uuid;

#[tokio::main(basic_scheduler)]
async fn main() {
  env_logger::init();
  
  let twin = env::var("TWIN_INSTANCE").unwrap();
  info!("Current Twin: {}", twin);

  let host = env::var("MQTT_BROKER_ADDRESS").unwrap();
  let port = env::var("MQTT_BROKER_PORT").unwrap().parse::<u16>().unwrap();

  info!("Connecting to broker at {}:{}", host, port);

  let mut mqttoptions = MqttOptions::new(format!("twin-{}", twin), host, port);
  mqttoptions.set_keep_alive(30);

  let mut eloop = EventLoop::new(mqttoptions, 20).await;
  let mut tx = eloop.handle();


  connect_to_topics(tx.clone()).await;

  loop {
    match eloop.poll().await {
      Ok(stream) => {
        let (incoming, _outgoing) = stream;

        if incoming.is_some() {
          match incoming.unwrap() {
            Incoming::PubAck(ack) => {
              info!("{:?}", ack);
            },
            Incoming::SubAck(ack) => {
              info!("{:?}", ack);
            },
            Incoming::Publish(publish) => {
              let message = String::from_utf8(publish.payload.to_vec()).expect("Convert message payload");
              let topic = publish.topic;
              
              handle_message(topic, message);
            },
            Incoming::Disconnect => {
              info!("Connection aborted. Reconnecting...");
              connect_to_topics(tx.clone()).await;
            },
            _ => {}
          }
        }
      },
      Err(e) => { error!("{:?}", e); }
    }

    time::delay_for(Duration::from_millis(10)).await;
  }
}

fn get_qos(variable: &str) -> QoS {
  let qos_value = env::var(variable).unwrap().parse::<u8>().unwrap();

  match qos_value {
    0 => QoS::AtMostOnce,
    1 => QoS::AtLeastOnce,
    2 => QoS::ExactlyOnce,
    _ => QoS::AtMostOnce
  }
}
async fn connect_to_topics(tx: Sender<Request>) {
  task::spawn(async move {
    let twin = env::var("TWIN_INSTANCE").unwrap();
    let qos = get_qos("MQTT_INSTANCE_QOS");

    // loop {
    let topic = format!("{}/+/+", twin);
    info!("Refreshing topics for twin {} - Listen to {}", twin, topic);

    let subscription = Subscribe::new(topic, qos);
    let _ = tx.send(Request::Subscribe(subscription)).await;
  
      // time::delay_for(Duration::from_secs(30)).await;
    // }
  });
}

fn handle_message(topic: String, message: String) {
  let tokens: Vec<&str> = topic.as_str().split("/").collect();
  let source = tokens[2];
  info!("{} \"{}\"", source, message);

  let session = get_db_session();

  let response = session.query(format!(
    "INSERT INTO source_data (source, stamp, value, created_at) VALUES ({}, toTimestamp(now()), '{}', toTimestamp(now()))",
    source, message
  ));

  match response {
    Ok(_) => info!("Inserted data for source {}.", source),
    Err(_) => error!("Error inserting data for source {}.", source)
  }
}

