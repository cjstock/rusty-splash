use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, RgbaImage};
use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::splashes::Splashes;

pub fn monitors(window: &Window) -> Vec<PhysicalSize<u32>> {
    let monitors = window.available_monitors();
    monitors.into_iter().map(|monitor| monitor.size()).collect()
}

#[derive(Debug, Default)]
pub struct TileParams {
    dims: (u32, u32),
    image_res: (u32, u32),
    image_adjust: (u32, u32),
}

pub fn find_optimal_tile(
    image_res: (u32, u32),
    container_res: (u32, u32),
    minimum_image_count: u32,
    minimum_image_dims: (u32, u32),
) -> Option<TileParams> {
    let mut c = 0;
    let mut best_fit: Option<TileParams> = None;
    loop {
        let x = calculate_tile_x_bias(image_res, container_res, c);
        let y = calculate_tile_y_bias(image_res, container_res, c);
        let mut temp_best: TileParams = TileParams::default();

        if x.image_adjust.1 > y.image_adjust.0 {
            temp_best = y;
        } else {
            temp_best = x;
        }

        if (temp_best.dims.0 * temp_best.dims.1 > minimum_image_count)
            && temp_best.image_res >= minimum_image_dims
        {
            best_fit = Some(temp_best);
            break;
        }

        if temp_best.image_res < minimum_image_dims {
            break;
        }
        c += 1;
    }
    best_fit
}

pub fn calculate_tile_y_bias(
    image_res: (u32, u32),
    container_res: (u32, u32),
    c: u32,
) -> TileParams {
    let image_res = (image_res.0 as f32, image_res.1 as f32);
    let container_res = (container_res.0 as f32, container_res.1 as f32);
    let fit_y = container_res.1 / image_res.1;
    let target_fit_y = fit_y.ceil() + (c as f32);
    let new_image_y = container_res.1 / target_fit_y;
    let image_ar = image_res.0 / image_res.1;
    let new_image_x = image_ar * new_image_y;
    let fit_x = container_res.0 / new_image_x;
    let overfit = fit_x.ceil();
    let overfit_error = fit_x.ceil() - fit_x;
    let pixel_x_error = overfit_error * new_image_x;
    let x_adjust = pixel_x_error / overfit;
    TileParams {
        dims: (overfit as u32, target_fit_y as u32),
        image_res: (new_image_x.round() as u32, new_image_y.round() as u32),
        image_adjust: (x_adjust.ceil() as u32, 0),
    }
}
pub fn calculate_tile_x_bias(
    image_res: (u32, u32),
    container_res: (u32, u32),
    c: u32,
) -> TileParams {
    let image_res = (image_res.0 as f32, image_res.1 as f32);
    let container_res = (container_res.0 as f32, container_res.1 as f32);
    let fit_x = container_res.0 / image_res.0;
    let target_fit_x = fit_x.ceil() + (c as f32);
    let new_image_x = container_res.0 / target_fit_x;
    let image_ar = image_res.0 / image_res.1;
    let new_image_y = new_image_x / image_ar;
    let fit_y = container_res.1 / new_image_y;
    let overfit = fit_y.ceil();
    let overfit_error = fit_y.ceil() - fit_y;
    let pixel_y_error = overfit_error * new_image_y;
    let y_adjust = pixel_y_error / overfit;
    TileParams {
        dims: (target_fit_x.round() as u32, overfit.round() as u32),
        image_res: (new_image_x.round() as u32, new_image_y.round() as u32),
        image_adjust: (0, y_adjust.ceil() as u32),
    }
}

pub fn adjust_images(images: &mut Vec<DynamicImage>, params: TileParams) {
    let len = images.len();
    let temp = Arc::new(Mutex::new(images.clone()));

    let mut threads = vec![];

    for i in 0..len {
        threads.push(thread::spawn({
            let clone = Arc::clone(&temp);
            move || {
                let mut t = clone.lock().unwrap();
                let image = &t[i];
                let image = image.resize(
                    params.image_res.0,
                    params.image_res.1,
                    image::imageops::FilterType::Lanczos3,
                );
                let image = image.crop_imm(
                    params.image_adjust.0 / 2,
                    params.image_adjust.1 / 2,
                    params.image_res.0 - (params.image_adjust.0),
                    params.image_res.1 - (params.image_adjust.1),
                );
                t[i] = image;
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}

pub fn build_tile(splashes: Vec<PathBuf>, monitor: (u32, u32)) {
    if !splashes.is_empty() {
        let mut images: Vec<DynamicImage> = vec![];
        for splash in &splashes {
            images.push(image::open(splash).unwrap());
        }
        let image_dims = images.first().unwrap().dimensions();
        println!("Calculating optimal tile...");
        let tile_params = find_optimal_tile(
            image_dims,
            monitor,
            images.len().try_into().unwrap(),
            (0, 0),
        );
        println!("Done");
        if let Some(params) = tile_params {
            let adjusted: Vec<DynamicImage> = images
                .iter_mut()
                .map(|image| {
                    println!("Resizing...");
                    let image = image.resize(
                        params.image_res.0,
                        params.image_res.1,
                        image::imageops::FilterType::Lanczos3,
                    );
                    println!("Cropping...");
                    image.crop_imm(
                        params.image_adjust.0 / 2,
                        params.image_adjust.1 / 2,
                        params.image_res.0 - (params.image_adjust.0),
                        params.image_res.1 - (params.image_adjust.1),
                    )
                })
                .collect();
            println!("Done");
            let mut new_image: RgbaImage = ImageBuffer::new(
                params.dims.0 * (params.image_res.0 - params.image_adjust.0),
                params.dims.1 * (params.image_res.1 - params.image_adjust.1),
            );
            let mut count: usize = 0;
            for i in 0..params.dims.1 {
                for j in 0..params.dims.0 {
                    let index = count.rem_euclid(adjusted.len());
                    let x = j * adjusted[index].dimensions().0;
                    let y = i * adjusted[index].dimensions().1;
                    new_image.copy_from(&adjusted[index], x, y).unwrap();
                    count += 1;
                }
            }

            let mut tile_path = splashes.first().unwrap().clone();
            tile_path.pop();
            tile_path.push("tiled.jpg");
            let _ = new_image.save_with_format(tile_path, image::ImageFormat::Jpeg);
        }
    }
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

#[test]
fn tile() {
    let splash_data = Splashes::new();
    let mut draven_path = splash_data.save_dir.clone();
    draven_path.push("Draven_0.jpg");

    let splashes = vec![draven_path];

    build_tile(splashes, (3840, 1600));
}
