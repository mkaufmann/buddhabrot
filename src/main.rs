extern crate chrono;

mod buddha;
use self::buddha::*;

mod render;
use self::render::*;

use std::sync::mpsc;
use std::thread;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn collect(point: ImaginaryNumber, iterations: u64, histogram: &mut Histogram) {
    let mut current = ImaginaryNumber::new(0.0, 0.0);
    for _ in 0..iterations {
        current = current * current + point;
        histogram.add_to_histogram(current.real, current.imaginary);
    }
}

fn construct_file_name(prefix: &str, suffix: &str) -> String {
    let now = chrono::Utc::now();
    format!("{}{}{}", prefix, now.to_string().replace(":","-").replace(" UTC",""), suffix)
}

fn main() {
    // Buddha sampling constants
    let real_range = (-2.0, 1.0);
    let imaginary_range = (-1.4, 1.4);
    let limit = 12000;
    let bailout = 4.0;
    let min_iterations = 5;

    // Image constants
    let resolution = (1920, 1920);

    // Runtime constants
    let num_cores = 4;
    let chunk_size = 20000;
    let base_seed = 1988;

    let (tx, rx) = mpsc::channel();

    for i in 0..num_cores {
        let thread_tx = tx.clone();
        let mut random_points = buddha::rand::ImaginaryNumberSource::new(real_range, imaginary_range, base_seed + i);
        thread::spawn(move || {
            loop {
                // Find qualifying points
                let mut points = vec![];
                while points.len() < chunk_size {
                    let point = random_points.sample();

                    escapes(point, limit, bailout)
                        .filter(|&iteration| iteration > min_iterations)
                        .map(|iteration| points.push((point, iteration)));
                }

                if thread_tx.send(points).is_err() {
                    return;
                }
            }
        });
    }

    // Receive point lists and store them and update image
    let file_name = &construct_file_name("output/points_", ".json");
    let path = Path::new(file_name);
    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open '{}': {}", file_name, why.description()),
        Ok(file) => file,
    };

    let mut histogram = Histogram::new(resolution.0, resolution.1, -2.0, 2.0, -2.0, 2.0);
    let mut count = 0;
    for points in rx {
        for (point, iterations) in points {
            collect(point, iterations, &mut histogram);
            if iterations > 5000 {
                file.write(format!("{{ point: [{},{}], iterations: {}}}\n", point.real, point.imaginary, iterations).as_bytes())
                    .unwrap();
            }
        }

        count += 1;
        if (count % 50) == 0 {
            println!("Saving");
            save_to_image("output/test.png", &mut histogram.data, resolution).unwrap();
        }
        if count == 10000 {
            return;
        }
    }
}
