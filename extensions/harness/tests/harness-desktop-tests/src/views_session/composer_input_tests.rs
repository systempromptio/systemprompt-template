use harness_desktop::app::settings::DeliveryPref;
use harness_desktop::views::composer::input::delivery;
use harness_engine::Delivery;

#[test]
fn enter_uses_the_preference_and_ctrl_enter_the_other_delivery() {
    assert_eq!(delivery(DeliveryPref::Steer, false), Delivery::Steer);
    assert_eq!(delivery(DeliveryPref::Steer, true), Delivery::Queue);
    assert_eq!(delivery(DeliveryPref::Queue, false), Delivery::Queue);
    assert_eq!(delivery(DeliveryPref::Queue, true), Delivery::Steer);
}
