use tokio::stream::StreamExt;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::{task,time};
use rumq_client::{eventloop,MqttEventLoop,MqttOptions, QoS, Request, Subscribe, Unsubscribe, PacketIdentifier};
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

  let mut mqttoptions = MqttOptions::new(format!("twin-{}", twin), host, port);
  mqttoptions.set_keep_alive(5).set_throttle(Duration::from_secs(1));

  let (mut tx,rx) = channel(10);
  let mut eloop = eventloop(mqttoptions, rx);

  listen_topics(&mut eloop, &mut tx).await;
}

async fn connect_to_topics(mut tx: Sender<Request>) {
  task::spawn(async move {
    let twin = env::var("TWIN_INSTANCE").unwrap();

    let mut subscribed_to: Vec<String> = Vec::new();

    loop {
      info!("Refreshing topics for twin {}", twin);

      let topics = get_instance_topics();
      info!("Listening to {} topics.", topics.len());

      let (subscribe_list, unsubscribe_list) = get_subscribe_and_unsubscribe_lists(&subscribed_to, &topics);

      if !unsubscribe_list.is_empty() {
        for topic in unsubscribe_list.clone() {
          match subscribed_to.binary_search(&topic) {
            Ok(index) => { subscribed_to.remove(index); },
            Err(e) => { error!("Error finding topic to unsubscribe: {}", e); }
          }
        }
  
        // TODO: Rumq does not support unsubscribe as of 17/06/2020.
        // Refactor and include when possible.
        // 
        //   let _ = tx.send(Request::Unsubscribe(Unsubscribe {
        //     pkid: PacketIdentifier(0),
        //     topics: unsubscribe_list
        //   })).await;
      }

      if !subscribe_list.is_empty() {
        info!("Adding {} topics.", subscribe_list.len());
        for topic in subscribe_list {
          subscribed_to.push(topic.clone());
          let subscription = Subscribe::new(topic, QoS::AtLeastOnce);
          let _ = tx.send(Request::Subscribe(subscription)).await;
        }
      }

      subscribed_to.sort_unstable();
  
      time::delay_for(Duration::from_secs(30)).await;
    }
  });
}

fn get_subscribe_and_unsubscribe_lists(subscribed_to: &Vec<String>, current_topics: &Vec<String>) -> (Vec<String>, Vec<String>) {
  let mut subscribe_list: Vec<String> = Vec::new();
  let mut unsubscribe_list: Vec<String> = Vec::new();

  for topic in current_topics {
    if !subscribed_to.contains(topic) {
      subscribe_list.push((*topic).clone());
    }
  }

  for topic in subscribed_to {
    if !current_topics.contains(topic) {
      unsubscribe_list.push((*topic).clone());
    }
  }

  (subscribe_list, unsubscribe_list)
}

fn get_twin_elements() -> Vec<Element> {
  let session = get_db_session();
  let twin = env::var("TWIN_INSTANCE").unwrap();

  let element_rows = session.query(format!("SELECT * FROM element WHERE twin = {}", twin))
    .expect("Elements from twin")
    .get_body().unwrap()
    .into_rows().unwrap();
  
  let mut elements: Vec<Element> = Vec::new();
 
  if !element_rows.is_empty() {
    for row in element_rows {
      match Element::try_from_row(row.clone()) {
        Ok(element) => elements.push(element),
        Err(_) => {}
      }
    }
  }

  info!("Found {} elements in twin.", elements.len());

  return elements; 
}

fn get_sources(elements: Vec<Element>) -> Vec<Source> {
  let session = get_db_session();
  let mut sources: Vec<Source> = Vec::new();

  let mut item_ids: Vec<String> = Vec::new();
  for element in elements {
    item_ids.push(element.id.to_string());
  }
  let element_ids = item_ids.join(",");

  let source_rows = session.query(format!("SELECT * FROM source WHERE element IN ({})", element_ids))
    .expect("Sources from element")
    .get_body().unwrap()
    .into_rows().unwrap();
  
  if !source_rows.is_empty() {
    for row in source_rows {
      match Source::try_from_row(row.clone()) {
        Ok(source) => sources.push(source),
        Err(_) => {}
      }
    }
  }

  info!("Found {} sources in twin elements.", sources.len());

  return sources;
}

fn get_instance_topics() -> Vec<String> {
  let elements = get_twin_elements();
  let sources = get_sources(elements);
  
  let mut subscription_list: Vec<String> = Vec::new();
  for source in sources {
    subscription_list.push(source.data_topic());
  }
  
  subscription_list
}

async fn listen_topics(eloop: &mut MqttEventLoop, tx: & Sender<Request>) {  
  connect_to_topics(tx.clone()).await;

  loop {
    let mut stream = eloop.connect().await.unwrap(); // TODO handle connection error. If broker doesn't exist, it crashes here.

    while let Some(item) = stream.next().await {
      match item {
        rumq_client::Notification::Puback(ack) => {
          info!("{:?}", ack);
        },
        rumq_client::Notification::Suback(ack) => {
          info!("{:?}", ack);
        },
        rumq_client::Notification::Publish(publish) => {
          let message = String::from_utf8(publish.payload).expect("Convert message payload");
  
          let topic = publish.topic_name;
  
          handle_message(topic, message);
        },
        rumq_client::Notification::Abort(_) => {
          info!("Connection aborted.");
        },
        _ => handle_message_error(item)
      }
    }
    time::delay_for(Duration::from_secs(1)).await;
  }
}

fn handle_message(topic: String, message: String) {
  info!("{}:: {}", topic, message);

  let session = get_db_session();

  let response = session.query(format!(
    "INSERT INTO source_data (source, stamp, value, created_at) VALUES ({}, toTimestamp(now()), '{}', toTimestamp(now()) )",
    topic, message
  ));

  match response {
    Ok(_) => info!("Inserted data for source {}.", topic),
    Err(_) => error!("Error inserting data for source {}.", topic)
  }
}

fn handle_message_error(notification: rumq_client::Notification) {
  error!("{:?}", notification);
}
