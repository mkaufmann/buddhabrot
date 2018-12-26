use std::fmt;
use std::ops::{Add, Mul};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ImaginaryNumber {
    pub real: f64,
    pub imaginary: f64,
}

impl ImaginaryNumber {
    pub fn new(real: f64, imaginary: f64) -> ImaginaryNumber {
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

pub mod rand {
    extern crate rand;

    use rand::distributions::Uniform;
    use rand::prelude::*;

    pub struct ImaginaryNumberSource {
        rng: SmallRng,
        real: Uniform<f64>,
        imaginary: Uniform<f64>,
    }

    impl ImaginaryNumberSource {
        pub fn new(real_range: (f64, f64), imaginary_range: (f64, f64), seed: u64) -> ImaginaryNumberSource {
            ImaginaryNumberSource {
                rng: rand::rngs::SmallRng::seed_from_u64(seed),
                real: Uniform::from(real_range.0..real_range.1),
                imaginary: Uniform::from(imaginary_range.0..imaginary_range.1),
            }
        }

        pub fn sample(&mut self) -> super::ImaginaryNumber {
            super::ImaginaryNumber::new(self.real.sample(&mut self.rng), self.imaginary.sample(&mut self.rng))
        }
    }
}

fn definitely_inside_mandelbrot(point: ImaginaryNumber) -> bool {
    let squared_imaginary = point.imaginary.powf(2.0);

    // Period2 Bulb
    let inside_p2_bulb = ((point.real+1.0).powf(2.0)+squared_imaginary) <= (1.0/16.0);
    if inside_p2_bulb {
        return true;
    }

    // Cardoid
    let p = ((point.real - 0.25).powf(2.0)+squared_imaginary).sqrt();
    let inside_cardiod = point.real <= p - 2.0*p.powf(2.0) + 0.25;
    if inside_cardiod {
        return true;
    }

    false
}

pub fn escapes(point: ImaginaryNumber, limit: u64, bailout: f64) -> Option<u64> {
    // Points that are inside the mandelbrot set would never escape, thus we use some quick
    // check to cheaply detect the biggest areas in the mandelbrot set without having to
    // iterate until the limit.
    if definitely_inside_mandelbrot(point) {
        return Option::None;
    }

    let mut current = ImaginaryNumber::new(0.0, 0.0);

    // We detect cycles by remembering one `reference point` and then checking all new points
    // against this point. For one start point, the cycle can only start after some
    // iteration. We thus update the `reference point` in increasing intervals and thus are able
    // to detect cycles up to the length of the current interval. This is far cheaper compared to
    // maintaining a map.
    let mut has_cycle = {
        let mut reference_point = current;
        let mut next_reference_update = 4;
        move |iteration, current| {
            if current == reference_point {
                return true;
            }
            if iteration == next_reference_update {
                reference_point = current;
                next_reference_update *= 2;
            }
            return false;
        }
    };

    for i in 0..limit {
        current = current * current + point;
        if (current.real * current.real) + (current.imaginary * current.imaginary) > bailout {
            return Option::Some(i);
        }
        if has_cycle(i, current) {
            return Option::None;
        }
    }

    return Option::None;
}
