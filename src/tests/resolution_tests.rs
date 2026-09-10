use crate::daemon::service::Resolution;

#[test]
fn resolution_ratio_str_correct() {
    assert_eq!(
        Resolution::new(1920_u64, 1080_u64).ratio_str(),
        String::from("16:9")
    );
    assert_eq!(
        Resolution::new(3840_u64, 2160_u64).ratio_str(),
        String::from("16:9")
    );
    assert_eq!(
        Resolution::new(77_u64, 143_u64).ratio_str(),
        String::from("7:13")
    );
    assert_eq!(
        Resolution::new(0_u64, 0_u64).ratio_str(),
        String::from("0:0")
    );
    assert_eq!(
        Resolution::new(0_u64, 42_u64).ratio_str(),
        String::from("0:1")
    );
    assert_eq!(
        Resolution::new(42_u64, 0_u64).ratio_str(),
        String::from("1:0")
    );
    assert_eq!(
        Resolution::new(u64::MAX, u64::MAX).ratio_str(),
        String::from("1:1")
    );
}
