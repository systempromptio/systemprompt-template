use systemprompt_engine::run::Delivery;
use systemprompt_harness_views::app::settings::DeliveryPref;
use systemprompt_harness_views::composer::input::delivery;

#[test]
fn enter_uses_the_preference_and_ctrl_enter_the_other_delivery() {
    assert_eq!(delivery(DeliveryPref::Steer, false), Delivery::Steer);
    assert_eq!(delivery(DeliveryPref::Steer, true), Delivery::Queue);
    assert_eq!(delivery(DeliveryPref::Queue, false), Delivery::Queue);
    assert_eq!(delivery(DeliveryPref::Queue, true), Delivery::Steer);
}
