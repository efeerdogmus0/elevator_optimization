#[cfg(test)]
use super::*;

#[test]
fn read() {
    let parameters = MotorParameters::from_file("param/motor_test_parameters.yaml").unwrap();
    println!("{:?}", parameters);
}