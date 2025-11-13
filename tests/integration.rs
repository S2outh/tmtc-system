use telemetry_system::*;

#[derive(TMValue)]
struct TestValue {
    val: u32
}

#[derive(TMValue)]
struct TestVector {
    x: i16,
    y: f32,
    z: TestValue
}

#[beacon(TestBeacon)]
enum TestBeaconDefinition {
    #[tmv(can_topic = 12, nats_topic = "telemetry.first_value")]
    FirstTMValue(u32),
    #[tmv(can_topic = 1, nats_topic = "telemetry.second_value")]
    SecondTMValue(TestValue),
    #[tmv(can_topic = 3, nats_topic = "telemetry.third_value")]
    ThirdTMValue(TestVector),
}

#[test]
fn test() {
    let mut beacon = TestBeacon::new();
    println!("len {}", beacon.bytes().len());
    println!("len {:?}", TestBeaconDefinition::TestBeacon_sizes);
    beacon.insert(TestBeaconDefinition::SecondTMValue(TestValue { val: 3 }));
    beacon.insert(TestBeaconDefinition::FirstTMValue(1234));
    beacon.insert(TestBeaconDefinition::ThirdTMValue(TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }}));

    println!("test");
}
