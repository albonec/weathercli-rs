use rand::distributions::Alphanumeric;
use rand::Rng;

#[test]
fn test_write_read() {
    let key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let value: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    weather_core::local::r#mod::write(&key, &value);
    let r = weather_core::local::r#mod::read(&key);
    assert!(r.is_some());
    assert_eq!(r.unwrap(), value);
}
