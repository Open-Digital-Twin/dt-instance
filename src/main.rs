extern crate rumqtt;
extern crate chrono;
extern crate text_io;
use text_io::read;
use rumqtt::{MqttClient,MqttOptions,QoS};
use chrono::prelude::*;
use std::{sync::Arc};

fn main() {


    let opts = MqttOptions::new("Rafael","localhost",1883);
    let (mut client, notifications) = MqttClient::start(opts).unwrap();
    let mut input : String = String::new();

    println!("Type the name of the topics you wish to subscribe to.\nType 'END' to start hearing on the chosen topics.");

    while input != String::from("END")
    {

        input = read!("{}");
        client.subscribe(&input,QoS::AtLeastOnce).unwrap();
         
    }

    println!("Hearing on chosen topics : \n");

    for notification in notifications {
    

    match notification
      {
        rumqtt::Notification::Publish(publish) =>
        {
            let payload = Arc::try_unwrap(publish.payload).unwrap();
            let topic = publish.topic_name;
            let text: String = String::from_utf8(payload).expect("cant convert string");
            let time_stamp = Local::now();
            println!("{} , On topic : {} : {}", time_stamp.format("%d/%m/%Y %H:%M:%S"),topic,text);
        }

        _ => println!("{:?}",notification)
     }

    
}
}


