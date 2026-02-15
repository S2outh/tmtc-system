#![feature(const_trait_impl)]
#![feature(const_cmp)]
use tmtc_system::*;

#[derive(TMValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestValue {
    val: u32,
}

#[derive(TMValue, Default, Clone, Copy)]
#[cfg_attr(feature = "ground", derive(serde::Serialize))]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue,
}

#[telemetry_definition(id = 0)]
mod telemetry {
    #[tmv(i64)]
    struct Timestamp;
    #[tmv(u32)]
    struct FirstTMValue;
    #[tmv(crate::TestValue)]
    struct SecondTMValue;
    #[tmm(id = 100)]
    mod some_other_mod {
        #[tmv(crate::TestVector)]
        struct ThirdTMValue;
    }
}

#[cfg(feature = "ground")]
extern crate alloc;

beacon!(
    TestBeacon,
    crate::telemetry,
    crate::telemetry::Timestamp,
    id = 0,
    values(FirstTMValue, SecondTMValue, some_other_mod::ThirdTMValue)
);

macro_rules! to_bytes {
    ($type: ty, $tm_value:ident) => {{
        let mut bytes = [0u8; <$type>::BYTE_SIZE];
        $tm_value.write(&mut bytes).unwrap();
        bytes
    }};
}

use test_beacon::TestBeacon;

fn crc_ccitt(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in bytes {
        crc ^= (*byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[test]
fn beacon_creation() {
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    beacon.first_tm_value = Some(first_value);
    beacon.second_tm_value = Some(second_value);
    beacon.some_other_mod_third_tm_value = Some(third_value);

    let sizes = [3, 1, 8, 4, (4), (2 + 4 + 4)];
    assert_eq!(beacon.to_bytes(&mut crc_ccitt).len(), sizes.iter().sum());
}

#[test]
fn beacon_insertion() {
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    beacon.first_tm_value = Some(first_value);
    beacon.second_tm_value = Some(second_value);
    beacon.some_other_mod_third_tm_value = Some(third_value);

    let bytes = beacon.to_bytes(&mut crc_ccitt);
    let crc = crc_ccitt(&bytes[3..]);
    // calculated with
    // https://www.crccalc.com/?crc=00, 00, 00, 00, 00, 00, 00, 00, D2, 04, 00, 00, 03, 00, 00, 00, 03, 00, 33, 33, 53, 40, 01, 00, 00, 00&method=CRC-16/CCITT-FALSE&datatype=hex&outtype=hex
    // assert_eq!(crc, 0x8798);

    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[1..3], crc.to_le_bytes());
    assert_eq!(bytes[12..16], first_value.to_le_bytes());
    assert_eq!(bytes[16..20], second_value.val.to_le_bytes());
    assert_eq!(bytes[20..22], third_value.x.to_le_bytes());
    assert_eq!(bytes[22..26], third_value.y.to_le_bytes());
    assert_eq!(bytes[26..30], third_value.z.val.to_le_bytes());
}

#[test]
fn beacon_insertion_id() {
    let mut id_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    id_beacon
        .insert_slice(telemetry::from_id(1).unwrap(), &to_bytes!(u32, first_value))
        .unwrap();
    id_beacon
        .insert_slice(
            telemetry::from_id(2).unwrap(),
            &to_bytes!(TestValue, second_value),
        )
        .unwrap();
    id_beacon
        .insert_slice(
            telemetry::from_id(100).unwrap(),
            &to_bytes!(TestVector, third_value),
        )
        .unwrap();

    beacon.first_tm_value = Some(first_value);
    beacon.second_tm_value = Some(second_value);
    beacon.some_other_mod_third_tm_value = Some(third_value);

    assert_eq!(
        id_beacon.to_bytes(&mut crc_ccitt),
        beacon.to_bytes(&mut crc_ccitt)
    );
}

#[test]
fn beacon_insertion_address() {
    let mut address_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: 1 },
    };

    address_beacon
        .insert_slice(
            telemetry::from_address("telemetry.first_tm_value").unwrap(),
            &to_bytes!(u32, first_value),
        )
        .unwrap();
    address_beacon
        .insert_slice(
            telemetry::from_address("telemetry.second_tm_value").unwrap(),
            &to_bytes!(TestValue, second_value),
        )
        .unwrap();
    address_beacon
        .insert_slice(
            telemetry::from_address("telemetry.some_other_mod.third_tm_value").unwrap(),
            &to_bytes!(TestVector, third_value),
        )
        .unwrap();

    beacon.first_tm_value = Some(first_value);
    beacon.second_tm_value = Some(second_value);
    beacon.some_other_mod_third_tm_value = Some(third_value);

    assert_eq!(
        address_beacon.to_bytes(&mut crc_ccitt),
        beacon.to_bytes(&mut crc_ccitt)
    );
}
