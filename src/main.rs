extern crate rumqtt;
extern crate chrono;
use rumqtt::{MqttClient,MqttOptions,QoS};
//use chrono::prelude;
use std::{thread, time::Duration, sync::Arc};


fn main() {


    let opts = MqttOptions::new("Rafael","localhost",1883);
    let (mut client, notifications) = MqttClient::start(opts).unwrap();
    client.subscribe("test",QoS::AtLeastOnce).unwrap();


    std::thread::spawn(move || {
    let dur = Duration::from_secs(1);
        
        for sender in 0..30{
        let g = sender.to_string();    
        client.publish("test", QoS::AtLeastOnce,false,g).unwrap();
        thread::sleep(dur);

    }});


    let mut it = 0;
    for notification in notifications {
    

    match notification
      {
        rumqtt::Notification::Publish(publish) =>
        {
            let payload = Arc::try_unwrap(publish.payload).unwrap();
            let text: String = String::from_utf8(payload).expect("cant convert string");
            println!("recieved message {}", text);
        }

        _ => println!("{:?}",notification)
     }

    it += 1;
    if it == 30{break};
    
}
}


