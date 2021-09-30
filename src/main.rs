use tokio::{task,time};

use rumqttc::{MqttOptions, QoS, EventLoop, AsyncClient, Request, Subscribe, Publish, Incoming, Outgoing, Event};
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

#[tokio::main]
async fn main() {
  env_logger::init();
  
  let twin = env::var("TWIN_INSTANCE").unwrap();
  info!("Current Twin: {}", twin);

  let host = env::var("MQTT_BROKER_ADDRESS").unwrap();
  let port = env::var("MQTT_BROKER_PORT").unwrap().parse::<u16>().unwrap();

  info!("Connecting to broker at {}:{}", host, port);

  let mut mqttoptions = MqttOptions::new(format!("twin-{}", twin), host, port);
  mqttoptions.set_keep_alive(30);

  match env::var("TWIN_INSTANCE_MAX_PACKET_SIZE") {
    Ok(max_size) => {
      info!("Set max packet size to {} KB.", max_size);
      mqttoptions.set_max_packet_size(max_size.parse::<usize>().unwrap(), max_size.parse::<usize>().unwrap());
    },
    Err(_) => {
      info!("Using default packet size of {} KB.", mqttoptions.max_packet_size());
    }
  }

  let (mut _client, mut eloop) = AsyncClient::new(mqttoptions, 20);
  let tx = eloop.handle();

  loop {
    match eloop.poll().await {
      Ok(event) => {
        match event {
          Event::Incoming(packet) => {
            match packet {
              Incoming::ConnAck(_ack) => {
                connect_to_topics(tx.clone()).await;
              },
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
                info!("Connection disconnected. Reconnecting...");
                connect_to_topics(tx.clone()).await;
              },
              _ => {
                error!("Unhandled incoming.");
              }
            }
          },
          _ => {}
        }        
      },
      Err(e) => {
        error!("MQTT ERROR: {:?}", e);
        time::sleep(Duration::from_millis(150)).await;
      }
    }

//     time::sleep(Duration::from_millis(1)).await;
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

    let topic = env::var("MQTT_SUBSCRIBED_TOPIC").unwrap();
    info!("Refreshing topics for twin {} - Listen to {}", twin, topic);

    let subscription = Subscribe::new(topic, qos);
    let _ = tx.send(Request::Subscribe(subscription)).await;
  });
}

fn handle_message(topic: String, message: String) {
  let tokens: Vec<&str> = topic.as_str().split("/").collect();
  let source = tokens[2];
  info!("{} \"{}\"", source, message);

// TEMP: remove saving to db.
//   let session = get_db_session();

//   let response = session.query(format!(
//     "INSERT INTO source_data (source, stamp, value, created_at) VALUES ({}, toTimestamp(now()), '{}', toTimestamp(now()))",
//     source, message
//   ));

//   match response {
//     Ok(_) => info!("Inserted data for source {}.", source),
//     Err(_) => error!("Error inserting data for source {}.", source)
//   }
}

