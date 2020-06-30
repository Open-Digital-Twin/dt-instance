# dt-instance

![Rust](https://github.com/Open-Digital-Twin/dt-instance/workflows/Rust/badge.svg)


### Mosquitto MQTT Broker

Configuration file is defined in `mosquitto.conf`. Options available in the [Manual page](https://mosquitto.org/man/mosquitto-conf-5.html).


### MQTT QoS
Environment variable `MQTT_INSTANCE_QOS` must be defined.

```
pub enum QoS {
  AtMostOnce = 0,
  AtLeastOnce = 1,
  ExactlyOnce = 2
}
```
