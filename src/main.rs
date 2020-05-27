use tokio::stream::StreamExt;
use tokio::sync::mpsc::{channel, Sender};
use tokio::{task,time};
use text_io::read;
use rumq_client::{eventloop,MqttEventLoop,MqttOptions, QoS, Request, Subscribe};
use chrono::prelude::*;
use std::time::Duration;

#[tokio::main(basic_scheduler)]
async fn main() {
  // TODO: Connect to DB, from parameters in .env

  // TODO: receive connection details as parameters.
  let (tx,rx) = channel(10);
  let mut mqttoptions = MqttOptions::new("ELE CONECTOU SIM", "localhost", 1883);

  mqttoptions.set_keep_alive(5).set_throttle(Duration::from_secs(1));
  let mut eloop = eventloop(mqttoptions, rx);

    let sub_list = get_instance_topics();

    task::spawn(async move{
    instance_sub_to_topics(tx, sub_list).await;
    time::delay_for(Duration::from_secs(3)).await;
    
    });
  
    listen_topics(&mut eloop).await;
}


  fn get_instance_topics() -> Vec<String> {
  // TODO: Get array of topics from DB.
  // for each topic, client.subscribe 
  // QoS is received as main parameter.
   println!("Type the name of the topics you wish to subscribe to.\nType 'END' to start hearing on the chosen topics.");

    let mut sub_list: Vec<String> = Vec::new();
    let mut input : String = String::new();

      while input != String::from("END")
      {
      input = read!("{}");
      sub_list.push(String::from(&input));
      }

      sub_list
}


async fn instance_sub_to_topics(mut tx: Sender<Request>, sub_list : Vec<String>) {

  for i in 0 .. sub_list.len() - 1{
  let subscription = Subscribe::new(String::from(&sub_list[i]), QoS::AtLeastOnce);
  let _ = tx.send(Request::Subscribe(subscription)).await;
  }

  time::delay_for(Duration::from_secs(100)).await;
}



async fn listen_topics(eloop: &mut MqttEventLoop) {
  
  println!("Hearing topics: \n");
  let mut stream = eloop.connect().await.unwrap();
  while let Some(item) = stream.next().await 
  {
      match item 
      {
      rumq_client::Notification::Publish(publish) =>{
      let message =String::from_utf8(publish.payload).expect("cant convert");
      let topic = publish.topic_name;
      let time_stamp = Local::now();

        handle_message(topic, message, time_stamp);
      }

      _ => handle_message_error(item)
    }
  }
}

fn handle_message(topic: String, message: String, time_stamp: chrono::DateTime<chrono::Local>) {
  println!("{} , On topic : {} : {}", time_stamp.format("%d/%m/%Y %H:%M:%S"), topic, message);
}

fn handle_message_error(notification: rumq_client::Notification) {
  println!("{:?}",notification);
}


