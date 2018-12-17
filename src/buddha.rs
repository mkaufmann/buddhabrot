use std::fmt;
use std::ops::{Add, Mul};

#[derive(Debug, Copy, Clone)]
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

pub fn escapes(point: ImaginaryNumber, limit: u64, bailout: f64) -> Option<u64> {
    let mut current = ImaginaryNumber::new(0.0, 0.0);
    let mut count = 0;
    for _ in 0..limit {
        current = current * current + point;
        if (current.real * current.real) + (current.imaginary * current.imaginary) > bailout {
            return Option::Some(count);
        }
        count = count + 1;
    }

    // TODO: Find cycles efficiently (check if equal to past value, updating the value in doubling intervals)
    return Option::None;
}
