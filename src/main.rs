extern crate rumqtt;
extern crate chrono;
use rumqtt::{MqttClient,MqttOptions,QoS,Message};
//use chrono::prelude;
use std::{thread, time::Duration};


fn main() {


    let opts = MqttOptions::new("Rafael","localhost",1883);
    let (mut client, notifications) = MqttClient::start(opts).unwrap();
    client.subscribe("test",QoS::AtLeastOnce).unwrap();


    std::thread::spawn(move || {
    let mut _sender = 0;
    let dur = Duration::from_secs(1);
        
        for _sender in 0..30{
        client.publish("test", QoS::AtLeastOnce,false,"UwU").unwrap();
        thread::sleep(dur);

    }});


    let mut it = 0;
    for notification in notifications {
        println!("{:?}", notification);
        it += 1;
        if it == 30 {break};
    }

}
