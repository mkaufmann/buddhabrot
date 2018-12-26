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
use std::time::Instant;

fn collect(point: ImaginaryNumber, iterations: u64, histogram: &mut Histogram) {
    let mut current = ImaginaryNumber::new(0.0, 0.0);
    for _ in 0..iterations {
        current = current * current + point;
        histogram.add_to_histogram(current.real, current.imaginary);
    }
}

fn construct_file_name(prefix: &str, suffix: &str) -> String {
    let now = chrono::Utc::now();
    format!("{}{}{}", prefix, now.to_string().replace(":", "-").replace(" UTC", ""), suffix)
}

fn main() {
    // Buddha sampling constants
    let real_range = (-2.0, 1.0);
    let imaginary_range = (-1.4, 1.4);
    let limit = 20000000;
    let bailout = 4.0;
    let min_iterations = 1000000;
    let target_points_count = 10000000000;

    // Image constants
    let resolution = (1920, 1920);

    // Runtime constants
    let num_cores = 4;
    let chunk_iterations_size = 5000000;
    let base_seed = 1988;

    let (tx, rx) = mpsc::channel();

    for i in 0..num_cores {
        let thread_tx = tx.clone();
        let mut random_points = buddha::rand::ImaginaryNumberSource::new(real_range, imaginary_range, base_seed + i);
        thread::spawn(move || {
            loop {
                // Find qualifying points
                let mut points = vec![];
                let mut probed = 0u64;
                let mut iterations_sum = 0u64;
                while iterations_sum < chunk_iterations_size {
                    let point = random_points.sample();
                    escapes(point, limit, bailout).filter(|&iteration| iteration > min_iterations).map(|iteration| {
                        iterations_sum += iteration;
                        points.push((point, iteration))
                    });
                    probed += 1;
                }

                if thread_tx.send((points, probed)).is_err() {
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
    let mut total_probed = 0;
    let mut total_points = 0;
    let save_interval = target_points_count / 10;
    let mut save_points = save_interval;
    let start = Instant::now();
    for (points, probed) in rx {
        total_probed += probed;
        for (point, iterations) in points {
            collect(point, iterations, &mut histogram);
            total_points += iterations;
            if iterations > (limit / 10) {
                file.write(format!("{{ point: [{},{}], iterations: {}}}\n", point.real, point.imaginary, iterations).as_bytes())
                    .unwrap();
            }
        }

        if total_points >= save_points {
            println!("Saving {} from {}: {}s", total_points, total_probed, start.elapsed().as_secs());
            save_to_image("output/test.png", &mut histogram.data, resolution).unwrap();
            save_points += save_interval;
        }
        if total_points >= target_points_count {
            println!("Duration: {}s", start.elapsed().as_secs());
            println!("Found: {}, probed: {}", total_points, total_probed);
            return;
        }
    }
}
