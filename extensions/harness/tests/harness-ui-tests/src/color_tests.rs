use systemprompt_harness_shared::theme::{Color, contrast_ratio};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

#[test]
fn black_on_white_is_twenty_one() {
    assert!(close(
        contrast_ratio(Color::hex(0x00_00_00), Color::hex(0xFF_FF_FF)),
        21.0
    ));
    assert!(close(
        contrast_ratio(Color::hex(0xFF_FF_FF), Color::hex(0x00_00_00)),
        21.0
    ));
}

#[test]
fn identical_colours_have_ratio_one() {
    assert!(close(
        contrast_ratio(Color::hex(0x77_77_77), Color::hex(0x77_77_77)),
        1.0
    ));
}

#[test]
fn known_reference_pair_matches_wcag() {
    assert!(close(
        contrast_ratio(Color::hex(0x76_76_76), Color::hex(0xFF_FF_FF)),
        4.54
    ));
}

#[test]
fn translucent_foreground_is_composited_first() {
    let half_black = Color::hex(0x00_00_00).with_alpha(0.5);
    let ratio = contrast_ratio(half_black, Color::hex(0xFF_FF_FF));
    let solid = contrast_ratio(Color::hex(0x80_80_80), Color::hex(0xFF_FF_FF));
    assert!((ratio - solid).abs() < 0.05);
}

#[test]
fn mix_moves_toward_the_other_colour() {
    let mixed = Color::hex(0x00_00_00).mix(Color::hex(0xFF_FF_FF), 0.5);
    assert_eq!(mixed.rgb(), 0x80_80_80);
    assert_eq!(
        Color::hex(0x12_34_56)
            .mix(Color::hex(0xFF_FF_FF), 0.0)
            .rgb(),
        0x12_34_56
    );
}

#[test]
fn hex_round_trips_alpha() {
    assert_eq!(Color::hex(0x1a_2b_3c).to_hex(), "#1a2b3c");
    assert_eq!(Color::hex(0x1a_2b_3c).with_alpha(0.5).to_hex(), "#1a2b3c80");
}
