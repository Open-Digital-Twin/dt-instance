extern crate mosquitto_client as mosq;
use mosq::Mosquitto;

fn main() {
    let adress = String::from("localhost");

    let m = Mosquitto::new("test");
    m.connect(&adress, 1883).expect("can't connect");
}
