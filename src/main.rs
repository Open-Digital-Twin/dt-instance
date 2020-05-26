extern crate rumqtt;
extern crate chrono;
extern crate text_io;
use text_io::read;
use rumqtt::{MqttClient,MqttOptions,QoS};
use chrono::prelude::*;
use std::{sync::Arc};

fn main() {
  // TODO: Connect to DB, from parameters in .env

  // TODO: receive connection details as parameters.
  let opts = MqttOptions::new("Rafael","localhost",1883);

  let (client, notifications) = MqttClient::start(opts).unwrap();

  get_instance_topics(client);
  listen_topics(notifications);
}


fn get_instance_topics(mut client: rumqtt::MqttClient) {
  // TODO: Get array of topics from DB.
  // for each topic, client.subscribe
  // QoS is received as main parameter.
  let mut input : String = String::new();
  
  println!("Type the name of the topics you wish to subscribe to.\nType 'END' to start hearing on the chosen topics.");
  
  while input != String::from("END") {
    input = read!("{}");
    client.subscribe(&input,QoS::AtLeastOnce).unwrap();
  }

  // TODO: return topics
}

fn listen_topics(notifications: rumqtt::Receiver<rumqtt::Notification>) {
  println!("Hearing topics: \n");

  for notification in notifications {
    match notification {
      rumqtt::Notification::Publish(publish) => {
        let payload = Arc::try_unwrap(publish.payload).unwrap();
        let topic = publish.topic_name;
        let message: String = String::from_utf8(payload).expect("cant convert string");
        let time_stamp = Local::now();

        handle_message(topic, message, time_stamp);
      }
      _ => handle_message_error(notification)
    }
  }
}

fn handle_message(topic: String, message: String, time_stamp: chrono::DateTime<chrono::Local>) {
  println!("{} , On topic : {} : {}", time_stamp.format("%d/%m/%Y %H:%M:%S"), topic, message);
}

fn handle_message_error(notification: rumqtt::Notification) {
  println!("{:?}",notification);
}


