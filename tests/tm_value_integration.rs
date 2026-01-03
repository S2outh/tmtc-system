#![feature(const_trait_impl)]
use tmtc_system::*;

#[derive(TMValue, Default, PartialEq, Debug, Clone, Copy)]
pub struct TestValue {
    val: u32
}

#[derive(TMValue, Default, PartialEq, Debug, Clone, Copy)]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue
}

//#[derive(TMValue, Default, PartialEq, Debug, Clone, Copy)]
//pub enum TestEnum {
//    #[default]
//    EmptyVar,
//    FirstVar(i16),
//    SecondVar(f32),
//    ThirdVar(TestValue)
//}

#[test]
fn tm_value_primitives() {
    let first_value = 4433u32;
    let first_value_bytes = first_value.to_bytes();
    let first_value_copy = u32::from_bytes(first_value_bytes);

    assert_eq!(first_value.to_le_bytes(), first_value_bytes);
    assert_eq!(first_value, first_value_copy);
}

#[test]
fn tm_value_structs() {
    let first_value = TestValue { val: 3 };
    let second_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};
    
    let first_value_bytes = first_value.to_bytes();
    let second_value_bytes = second_value.to_bytes();

    let first_value_copy = TestValue::from_bytes(first_value_bytes);
    let second_value_copy = TestVector::from_bytes(second_value_bytes);

    assert_eq!(first_value, first_value_copy);
    assert_eq!(second_value, second_value_copy);
}

#[test]
fn tm_value_arrays() {
    let first_value: [i32; 7] = [ 1, 2, 3, 4, 3, 2, 1 ];
    let first_value_bytes: [u8; 7*4] = first_value.to_bytes();
    let first_value_copy = <[i32; 7]>::from_bytes(first_value_bytes);

    assert_eq!(first_value, first_value_copy);
}

//#[test]
//fn tm_value_enums() {
//    let first_value = TestEnum::ThirdVar(TestValue { val: 42 });
//    let first_value_bytes: [u8; 1+4] = first_value.to_bytes();
//    let first_value_copy = TestEnum::from_bytes(first_value_bytes);
//
//    assert_eq!(first_value, first_value_copy);
//}
