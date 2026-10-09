use damage_numbers_core::{presentation::*, *};
fn measure(text: &[u8; 12], pixels: f32, _style: Style) -> Option<TextMetrics> {
    let width = text.iter().take_while(|&&b| b != 0).count() as f32 * pixels * 0.55;
    Some(TextMetrics {
        advance: width,
        ink: Rect {
            left: 0.0,
            top: pixels * 0.15,
            right: width,
            bottom: pixels * 0.85,
        },
    })
}
fn main() {
    for (style_index, style) in [Style::Wilds, Style::Classic, Style::Rise]
        .into_iter()
        .enumerate()
    {
        for (crit_index, critical) in [Critical::None, Critical::Positive, Critical::Negative]
            .into_iter()
            .enumerate()
        {
            for hz in 0..3 {
                for step in 0..9 {
                    let mut model = Model::default();
                    model.reset(1);
                    model.configure(Settings {
                        style,
                        ..Settings::default()
                    });
                    for i in 0..8 {
                        model.capture(Event {
                            key: HitKey {
                                epoch: 1,
                                serial: i + 1,
                                target: 2,
                                application: 0,
                            },
                            position: Vec3 {
                                x: (i as i32 % 3 - 1) as f32 * 0.03,
                                y: (i as i32 / 3 - 1) as f32 * 0.02,
                                z: 0.5,
                            },
                            total_damage: (i * 17 + 1) as i32,
                            physical_hzv: if hz == 0 {
                                -1
                            } else if hz == 1 {
                                44
                            } else {
                                45
                            },
                            source: Source::Attack,
                            owner: Owner::LocalPlayer,
                            critical,
                            bounced: false,
                            captured_at: 0.0,
                        });
                    }
                    model.update(0.01, true);
                    let view = View {
                        world_to_clip: [
                            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0,
                        ],
                        width: 1920.0,
                        height: 1080.0,
                        ..View::default()
                    };
                    let mut cache = Placement::default();
                    let mut out = [DrawNumber::default(); 128];
                    layout(&model, &view, 0.01, &mut out, &mut cache, measure);
                    let now = step as f32 * 0.1;
                    model.update(now, true);
                    let count = layout(&model, &view, now, &mut out, &mut cache, measure);
                    println!("{} {} {} {} {}", style_index, crit_index, hz, step, count);
                    for d in &out[..count] {
                        let values = [
                            d.position.x,
                            d.position.y,
                            d.offset.x,
                            d.offset.y,
                            d.star.x,
                            d.star.y,
                            d.slash.origin.x,
                            d.slash.origin.y,
                            d.slash.slope,
                            d.slash.extent,
                            d.slash.width,
                            d.slash.opacity,
                            d.pixels,
                            d.opacity,
                            d.advance,
                            d.unit,
                            d.classic_stroke,
                            d.ink.left,
                            d.ink.top,
                            d.ink.right,
                            d.ink.bottom,
                            d.solid.left,
                            d.solid.top,
                            d.solid.right,
                            d.solid.bottom,
                            d.reserve.left,
                            d.reserve.top,
                            d.reserve.right,
                            d.reserve.bottom,
                        ];
                        for value in values {
                            print!("{} ", value);
                        }
                        let end = d.text.iter().position(|&b| b == 0).unwrap();
                        println!(
                            "{} {} {}",
                            d.color,
                            d.key.serial,
                            std::str::from_utf8(&d.text[..end]).unwrap()
                        );
                    }
                }
            }
        }
    }
}
