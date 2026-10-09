//! Mirror the immutable r25 source/outcome and native camera fixtures with production portable functions.
use damage_numbers_core::{presentation::*, *};
fn main() {
    for owner in [
        Owner::LocalPlayer,
        Owner::LocalPalico,
        Owner::RemotePlayer,
        Owner::RemotePalico,
        Owner::Unknown,
    ] {
        for a in 0..5 {
            for b in 0..10 {
                for c in 0..5 {
                    println!(
                        "S {}",
                        source_outcome(a | (b << 8) | (c << 16), owner).map_or(255, |s| s as u8)
                    );
                }
            }
        }
    }
    for flag in 0..=255 {
        println!("C {}", critical_outcome(flag) as u8);
    }
    for kind in 0..=255 {
        for amount in [-1, 0, 1, 9999] {
            println!("D {}", u8::from(damage_outcome(kind, amount)));
        }
    }
    for camera in 0..6 {
        let mut v = [
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
        ];
        let mut p = v;
        match camera {
            1 => v[12] = 0.2,
            2 => {
                v[0] = 0.8;
                v[5] = 0.7;
                v[13] = -0.15;
            }
            3 => {
                p[11] = 0.2;
                p[15] = 0.8;
            }
            4 => p[0] = f32::NAN,
            5 => p[15] = -1.,
            _ => {}
        }
        for x in [-2., -1., 0., 0.7, 2.] {
            for y in [-1., 0., 1.] {
                for z in [-0.1, 0.5, 2.] {
                    let point = native_camera(&v, &p, [0, 0, 1280, 720], 1280., 720.)
                        .and_then(|view| project(Vec3 { x, y, z }, &view));
                    if let Some(point) = point {
                        println!("P 1 {} {}", point.x, point.y);
                    } else {
                        println!("P 0");
                    }
                }
            }
        }
    }
}
