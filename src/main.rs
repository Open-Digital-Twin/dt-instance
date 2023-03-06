use tokio::{task,time};

use rumqttc::{MqttOptions, QoS, AsyncClient, Incoming, Event};
//use rumqttc::{EventLoop, Publish, Outgoing,};
use rand::{distributions::Alphanumeric, Rng, thread_rng};
use chrono::Utc;

use std::time::Duration;
use std::env;

extern crate env_logger;

use log::{info, error};

#[macro_use]
#[allow(unused_imports)]
extern crate cdrs;

//#[macro_use]
extern crate cdrs_helpers_derive;
//use cdrs::query::*;
//use crate::cdrs::frame::TryFromRow; 

// mod common;
// use common::db::get_db_session;
// use common::models::twin::*;

//use uuid::Uuid;

#[tokio::main]
async fn main() {
  env_logger::init();
  
  // TEMP: twin instance name is now randomized
  // let twin = env::var("TWIN_INSTANCE").unwrap();

  let no_id: String = std::iter::repeat(())
    .map(|()| thread_rng().sample(Alphanumeric))
    .take(15).collect();
  
  let id: String = env::var("TWIN_INSTANCE_NAME").unwrap_or(no_id);
  info!("Current Twin: {}", id);
  let host = env::var("MQTT_BROKER_ADDRESS").unwrap();
  let port = env::var("MQTT_BROKER_PORT").unwrap().parse::<u16>().unwrap();
  let log_each = env::var("LOG_EACH").unwrap_or(1.to_string()).parse::<i32>().unwrap();

  info!("Connecting to broker at {}:{}", host, port);
  info!("Logging info every \"{}\" messages", log_each);

  let mut mqttoptions = MqttOptions::new(id, host, port);
  mqttoptions.set_keep_alive(Duration::from_secs(30));
  mqttoptions.set_clean_session(true);

  match env::var("TWIN_INSTANCE_MAX_PACKET_SIZE") {
    Ok(max_size) => {
      info!("Set max packet size to {} KB.", max_size);
      mqttoptions.set_max_packet_size(max_size.parse::<usize>().unwrap(), max_size.parse::<usize>().unwrap());
    },
    Err(_) => {
      info!("Using default packet size of {} KB.", mqttoptions.max_packet_size());
    }
  }

  let (client, mut eloop) = AsyncClient::new(mqttoptions, 20);
  let mut message_counter = 0;
  loop {
    match eloop.poll().await {
      Ok(event) => {
        match event {
          Event::Incoming(packet) => {
            match packet {
              Incoming::ConnAck(_ack) => {
                connect_to_topics(client.clone()).await;
              },
              Incoming::PubAck(ack) => {
                info!("{:?}", ack);
              },
              Incoming::SubAck(ack) => {
                info!("{:?}", ack);
              },
              Incoming::Publish(publish) => {
                message_counter += 1;
                //print!("message counter - {} message counter%10 - {} ", message_counter, message_counter%10);
                if message_counter % log_each == 0 {
                  let message = String::from_utf8(publish.payload.to_vec()).expect("Convert message payload");
                  let topic = publish.topic;
                  handle_message(topic, message);
                }
              },
              Incoming::Disconnect => {
                info!("Connection disconnected. Reconnecting...");
                connect_to_topics(client.clone()).await;
              },
              Incoming::PingResp => {},
              Incoming::PingReq => {},
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

async fn connect_to_topics(client: AsyncClient) {
  task::spawn(async move {
    let qos = get_qos("MQTT_INSTANCE_QOS");

    let topic = env::var("MQTT_SUBSCRIBED_TOPIC").unwrap();
    info!("Refreshing topics for twin - Listen to {}", topic);

    client.subscribe(topic, qos).await.unwrap();
  });
}

// fn add_to_db(source: String, message: String) {
//   let session = get_db_session();

//   let response = session.query(format!(
//     "INSERT INTO source_data (source, stamp, value, created_at) VALUES ({}, toTimestamp(now()), '{}', toTimestamp(now()))",
//     source, message
//   ));

//   match response {
//     Ok(_) => info!("Inserted data for source {}.", source),
//     Err(_) => error!("Error inserting data for source {}.", source)
//   }
// }

fn handle_message(topic: String, message: String) {
  let tokens: Vec<&str> = topic.as_str().split("_").collect();
  let source = tokens[tokens.len()-1];
  
  // info!("{} \"{}\"", source, message);
  let dt = Utc::now().to_string();
  let payloadparse: Vec<&str> = message.split(" ").collect();
  info!("received at {} - {} \"{}\" {}", dt, source, payloadparse[0], payloadparse[2]);

  // add_to_db(source, message);
}
