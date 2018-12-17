extern crate image;

pub mod buddha;
use self::buddha::*;

use std::sync::mpsc;
use std::thread;

struct Rasterizer {
    real_range: (f64, f64),
    imaginary_range: (f64, f64),
    resolution_x: u64,
    resolution_y: u64,
    offset_x: f64,
    offset_y: f64,
    step_x: f64,
    step_y: f64,
}

impl Rasterizer {
    fn new(real_range: (f64, f64), imaginary_range: (f64, f64), x: u64, y: u64) -> Rasterizer {
        let offset_x = 0.0 - real_range.0;
        let offset_y = 0.0 - imaginary_range.0;
        let step_x = (real_range.1 - real_range.0) / x as f64;
        let step_y = (imaginary_range.1 - imaginary_range.0) / y as f64;
        Rasterizer {
            real_range: real_range,
            imaginary_range: imaginary_range,
            resolution_x: x,
            resolution_y: y,
            offset_x: offset_x,
            offset_y: offset_y,
            step_x: step_x,
            step_y: step_y,
        }
    }

    fn rasterize(&self, point: ImaginaryNumber) -> (u64, u64) {
        let x = (point.real + self.offset_x) / self.step_x;
        let y = (point.imaginary + self.offset_y) / self.step_y;
        (x as u64, y as u64)
    }
}

fn draw(point: ImaginaryNumber, iterations: u64, rasterizer: &Rasterizer, data: &mut [u64]) {
    let mut current = ImaginaryNumber::new(0.0, 0.0);
    for _ in 0..iterations {
        current = current * current + point;
        let (x, y) = rasterizer.rasterize(current);
        //println!("{} => {}, {}", current, x, y);
        data[(x * rasterizer.resolution_y + y) as usize] += 1;
    }
}

fn to_image(data: &mut [u64], resolution: (u64, u64)) {
    let max = data.iter().max().unwrap();
    let img = image::ImageBuffer::from_fn(resolution.0 as u32, resolution.1 as u32, |x, y| {
        let raw = data[(x as u64 * resolution.1 + y as u64) as usize];
        let normalized = raw * 255 / max;
        image::Luma([normalized as u8])
    });
    img.save("test.png").unwrap();
}

fn main() {
    let resolution = (1920, 1920);
    // TODO: Fix this and avoid distortion and rotate
    let rasterizer = Rasterizer::new((-2.0, 2.0), (-2.0, 2.0), resolution.0, resolution.1);

    let limit = 12000;
    let bailout = 4.0;

    let mut data = vec![0u64; (rasterizer.resolution_x * rasterizer.resolution_y) as usize];

    let num_cores = 4;
    let (tx, rx) = mpsc::channel();

    for _ in 0..num_cores {
        let thread_tx = tx.clone();
        let mut random_points = buddha::rand::ImaginaryNumberSource::new(
            rasterizer.real_range,
            rasterizer.imaginary_range,
            1988,
        );
        thread::spawn(move || {
            loop {
                // Find qualifying points
                let mut points = vec![];
                while points.len() < 20000 {
                    let point = random_points.sample();

                    escapes(point, limit, bailout)
                        .filter(|&iteration| iteration > 5)
                        .map(|iteration| points.push((point, iteration)));
                }

                if thread_tx.send(points).is_err() {
                    return;
                }
            }
        });
    }

    // Receive point lists and store them and update image
    let mut count = 0;
    for points in rx {
        for (point, iterations) in points {
            draw(point, iterations, &rasterizer, &mut data);
        }

        count += 1;
        if (count % 50) == 0 {
            println!("Saving");
            to_image(&mut data, resolution);
        }
        if count == 10000 {
            return;
        }
    }
}
