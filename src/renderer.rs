use cgmath::Vector2;
use sdl2::{pixels::PixelFormatEnum, surface::SurfaceRef};

use crate::terrain::SIZE;

#[derive(Debug, Clone, Copy)]
pub struct RenderParams {
    pub origin: Vector2<f32>,
    pub axes: [Vector2<f32>; 3],
}

#[derive(Debug, Clone, Copy, Default)]
struct Span {
    min_y: u32,
    color: u32,
}

pub fn render_to(surf: &mut SurfaceRef, terrain: &Vec<u32>, params: &RenderParams) {
    assert_eq!(surf.pixel_format_enum(), PixelFormatEnum::ARGB8888);

    let ((width, height), pitch) = (surf.size(), surf.pitch());

    let num_pixels = width.checked_mul(height).unwrap();
    let mut pixels_transposed: Vec<u32> = vec![0xff2f2f2f; num_pixels as usize];
    let mut min_ys: Vec<u16> = vec![height as _; width as usize];

    let mut row_x = vec![Default::default(); SIZE];
    let mut row_y = vec![Default::default(); SIZE];

    let axes_0_15 = (params.axes[0] * 32768.0).cast::<i32>().unwrap();
    let axes_2_16 = (params.axes[2] * 65536.0).cast::<i32>().unwrap();

    // From foreground to background
    for ter_y in 0..SIZE {
        let ref terrain_row = terrain[ter_y * SIZE..][..SIZE];

        let row_origin = params.origin + params.axes[1] * (ter_y as f32 + 0.5);

        let row_origin_16 = (row_origin * 65536.0).cast::<i32>().unwrap();

        // Project a single row of a given heightmap onto the screen
        for ter_x in 0..SIZE {
            let mut projected_center = row_origin_16 + axes_0_15 * (ter_x as i32 * 2 + 1);
            let mut projected_end = row_origin_16 + axes_0_15 * (ter_x as i32 * 2);

            projected_center.y += (terrain_row[ter_x] >> 24) as i32 * axes_2_16.y;

            if projected_center.y < 0 {
                projected_center.y = 0;
            }
            if projected_end.x < 0 {
                projected_end.x = 0;
            }
            if projected_end.x > (width as i32) << 16 {
                projected_end.x = (width as i32) << 16;
            }

            row_x[ter_x] = projected_end.x >> 16 as u32;
            row_y[ter_x] = projected_center.y >> 16 as u32;
        }

        let mut screen_x = row_origin.x.max(0.0).min(width as _) as usize;

        if axes_0_15.x > 0 {
            for x in 0..SIZE {
                while screen_x < row_x[x] as usize {
                    let ref mut column_pixels =
                        pixels_transposed[screen_x * (height as usize)..][..height as usize];
                    let ref mut column_min_y = min_ys[screen_x];

                    while *column_min_y > row_y[x] as u16 {
                        *column_min_y -= 1;
                        column_pixels[(*column_min_y) as usize] = terrain_row[x];
                    }

                    screen_x += 1;
                }
            }
        } else {
            for x in 0..SIZE {
                while screen_x > row_x[x] as usize {
                    screen_x -= 1;

                    let ref mut column_pixels =
                        pixels_transposed[screen_x * (height as usize)..][..height as usize];
                    let ref mut column_min_y = min_ys[screen_x];

                    while *column_min_y > row_y[x] as u16 {
                        *column_min_y -= 1;
                        column_pixels[(*column_min_y) as usize] = terrain_row[x];
                    }
                }
            }
        }
    }

    // Output to the framebuffer
    let pixels = surf.without_lock_mut().unwrap().as_mut_ptr();

    // This transposition could be optimized using a cache-oblivious algorithm
    for y in 0..height {
        for x in 0..width {
            unsafe {
                *(pixels.offset((x * 4 + y * pitch) as isize) as *mut u32) =
                    *pixels_transposed.get_unchecked((y + x * height) as usize) | 0xff000000;
            }
        }
    }
}
