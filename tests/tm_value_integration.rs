#![feature(const_trait_impl)]
use tmtc_system::*;

#[derive(TMValue, Default, PartialEq, Debug, Clone, Copy)]
pub struct TestValue {
    val: Option::<u32>,
}

#[derive(TMValue, Default, PartialEq, Debug, Clone, Copy)]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue,
}

#[derive(TMValue, Default, PartialEq, Debug, Clone, Copy)]
pub enum TestEnum {
    #[default]
    EmptyVar,
    FirstVar(i16),
    SecondVar(f32),
    ThirdVar(TestValue),
}

macro_rules! to_bytes {
    ($type: ty, $tm_value:ident) => {{
        let mut bytes = [0u8; <$type>::BYTE_SIZE];
        $tm_value.write(&mut bytes).unwrap();
        bytes
    }};
}

#[test]
fn tm_value_primitives() {
    let first_value = 4433u32;
    let first_value_bytes = to_bytes!(u32, first_value);
    let first_value_copy = u32::read(&first_value_bytes).unwrap().1;

    assert_eq!(first_value.to_le_bytes(), first_value_bytes);
    assert_eq!(first_value, first_value_copy);
}

#[test]
fn tm_value_structs() {
    let first_value = TestValue { val: Some(3) };
    let second_value = TestVector {
        x: 3,
        y: 3.3,
        z: TestValue { val: Some(1) },
    };

    let first_value_bytes = to_bytes!(TestValue, first_value);
    let second_value_bytes = to_bytes!(TestVector, second_value);

    let first_value_copy = TestValue::read(&first_value_bytes).unwrap().1;
    let second_value_copy = TestVector::read(&second_value_bytes).unwrap().1;

    assert_eq!(first_value, first_value_copy);
    assert_eq!(second_value, second_value_copy);
}

#[test]
fn tm_value_arrays() {
    let first_value: [i32; 7] = [1, 2, 3, 4, 3, 2, 1];
    let first_value_bytes: [u8; 7 * 4] = to_bytes!([i32; 7], first_value);
    let first_value_copy = <[i32; 7]>::read(&first_value_bytes).unwrap().1;

    assert_eq!(first_value, first_value_copy);
}

#[test]
fn tm_value_enums() {
    let first_value = TestEnum::ThirdVar(TestValue { val: Some(42) });
    let first_value_bytes: [u8; 1 + 1 + 4] = to_bytes!(TestEnum, first_value);
    let first_value_copy = TestEnum::read(&first_value_bytes).unwrap().1;

    assert_eq!(first_value, first_value_copy);
}
