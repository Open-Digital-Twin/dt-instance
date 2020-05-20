extern crate rumqtt;
extern crate chrono;
extern crate text_io;
use text_io::read;
use rumqtt::{MqttClient,MqttOptions,QoS};
use chrono::prelude::*;
use std::{thread, time::Duration, sync::Arc};

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


    std::thread::spawn(move || {
    let dur = Duration::from_secs(1);
        
        for sender in 0..30{
        let g = sender.to_string();    
        client.publish("test", QoS::AtLeastOnce,false,g).unwrap();
        if sender % 3 == 0{client.publish("hello", QoS::AtLeastOnce,false,"something else").unwrap();}
        thread::sleep(dur);

    }});




    let mut it = 0;
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

    it += 1;
    if it == 30{break};
    
}
}


