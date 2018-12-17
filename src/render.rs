extern crate image;

pub struct Histogram {
    resolution_x: u64,
    resolution_y: u64,
    offset_x: f64,
    offset_y: f64,
    step_x: f64,
    step_y: f64,
    pub data: Vec<u64>,
}

impl Histogram {
    pub fn new(resolution_x: u32, resolution_y: u32, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Histogram {
        let offset_x = 0.0 - min_x;
        let offset_y = 0.0 - min_y;
        let step_x = (max_x - min_x) / resolution_x as f64;
        let step_y = (max_y - min_y) / resolution_y as f64;
        Histogram {
            resolution_x: resolution_x as u64,
            resolution_y: resolution_y as u64,
            offset_x: offset_x,
            offset_y: offset_y,
            step_x: step_x,
            step_y: step_y,
            data: vec![0; (resolution_x * resolution_y) as usize],
        }
    }

    fn rasterize(&self, x: f64, y: f64) -> (u64, u64) {
        let x = ((x + self.offset_x) / self.step_x) as u64;
        let y = ((y + self.offset_y) / self.step_y) as u64;

        assert!(x < self.resolution_x, "{} is larger then {}", x, self.resolution_x);
        assert!(y < self.resolution_y, "{} is larger then {}", y, self.resolution_y);
        (x as u64, y as u64)
    }

    pub fn add_to_histogram(&mut self, x: f64, y: f64) {
        let (x, y) = self.rasterize(x, y);
        self.data[(x * self.resolution_y + y) as usize] += 1;
    }
}

pub fn save_to_image(file_name: &str, data: &mut [u64], resolution: (u32, u32)) -> std::io::Result<()> {
    let max = data.iter().max().unwrap();
    let img = image::ImageBuffer::from_fn(resolution.0, resolution.1, |x, y| {
        let raw = data[x as usize * resolution.1 as usize + y as usize];
        let normalized = raw * 255 / max;
        image::Luma([normalized as u8])
    });
    img.save(file_name)
}
