extern crate image;
extern crate rand;

use rand::distributions::Uniform;
use rand::prelude::*;
use std::fmt;
use std::ops::{Add, Mul};

#[derive(Debug, Copy, Clone)]
struct ImaginaryNumber {
    real: f64,
    imaginary: f64,
}

impl ImaginaryNumber {
    fn new(real: f64, imaginary: f64) -> ImaginaryNumber {
        ImaginaryNumber {
            real: real,
            imaginary: imaginary,
        }
    }
}

impl Add for ImaginaryNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let real = self.real + rhs.real;
        let imaginary = self.imaginary + rhs.imaginary;
        ImaginaryNumber::new(real, imaginary)
    }
}

impl Mul for ImaginaryNumber {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let real = self.real * rhs.real - self.imaginary * rhs.imaginary;
        let imaginary = self.real * rhs.imaginary + rhs.real * self.imaginary;
        ImaginaryNumber::new(real, imaginary)
    }
}

impl fmt::Display for ImaginaryNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.real, self.imaginary)
    }
}

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

fn escapes(point: ImaginaryNumber, limit: u64, bailout: f64) -> u64 {
    let mut current = ImaginaryNumber::new(0.0, 0.0);
    let mut count = 0;
    for _ in 0..limit {
        current = current * current + point;
        if (current.real * current.real) + (current.imaginary * current.imaginary) > bailout {
            return count;
        }
        count = count + 1;
    }
    return limit;
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

fn to_image(data: &mut[u64], resolution: (u64, u64)) {
    let max = data.iter().max().unwrap();
    let img = image::ImageBuffer::from_fn(resolution.0 as u32, resolution.1 as u32, |x, y| {
        let raw = data[(x as u64*resolution.1+y as u64) as usize];
        let normalized = raw * 255 / max;
        image::Luma([normalized as u8])
    });
    img.save("test.png").unwrap();
}

fn main() {
    let resolution = (1920, 1920);
    // TODO: Fix this and avoid distortion and rotate
    let rasterizer = Rasterizer::new((-2.0, 2.0), (-2.0, 2.0), resolution.0, resolution.1);

    let mut imaginary_number_source = {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1988);
        let real_dist = Uniform::from(rasterizer.real_range.0..rasterizer.real_range.1);
        let imaginary_dist =
            Uniform::from(rasterizer.imaginary_range.0..rasterizer.imaginary_range.1);
        move || ImaginaryNumber::new(real_dist.sample(&mut rng), imaginary_dist.sample(&mut rng))
    };

    let limit = 8000;
    let bailout = 4.0;
    let mut points = vec![];

    println!("Searching");
    while points.len() < 10000000 {
        let point = imaginary_number_source();
        let iterations = escapes(point, limit, bailout);
        if (iterations != limit) && (iterations > 5) {
            points.push((point, iterations));
            //println!("point = {}, {}", point, rasterizer.rasterize(point).0);
        }
    }

    println!("Building picture");
    let mut data = vec![0u64; (rasterizer.resolution_x * rasterizer.resolution_y) as usize];
    for (point, iterations) in points {
        draw(point, iterations, &rasterizer, &mut data);
    }

    println!("Saving");
    to_image(&mut data, resolution);
}
