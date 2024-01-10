use image::{GenericImage, GenericImageView, ImageBuffer, RgbaImage};
use std::path::PathBuf;
use winit::{dpi::PhysicalSize, window::Window};

pub fn monitors(window: &Window) -> Vec<PhysicalSize<u32>> {
    let monitors = window.available_monitors();
    monitors.into_iter().map(|monitor| monitor.size()).collect()
}

pub fn merge_two(left_path: PathBuf, right_path: PathBuf) {
    let left_img = image::open(&left_path).unwrap();
    let right_img = image::open(right_path).unwrap();
    let (width, height) = (
        left_img.dimensions().0 + right_img.dimensions().0,
        left_img.dimensions().1,
    );
    let mut tiled_image: RgbaImage = ImageBuffer::new(width, height);
    tiled_image.copy_from(&left_img, 0, 0).unwrap();
    tiled_image
        .copy_from(&right_img, left_img.dimensions().0, 0)
        .unwrap();
    let mut merged_path = left_path.clone();
    merged_path.pop();
    merged_path.push("merged.jpg");
    let _ = tiled_image.save_with_format(merged_path, image::ImageFormat::Jpeg);
}

#[cfg(test)]
#[test]
fn merge() {
    use crate::splashes::Splashes;

    let splash_data = Splashes::new();
    let mut left = splash_data.save_dir.clone();
    let mut right = splash_data.save_dir.clone();
    left.push("Draven_0.jpg");
    right.push("Draven_1.jpg");

    merge_two(left, right);
}
