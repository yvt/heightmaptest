use cgmath::vec2;
use noise::NoiseFn;

pub const SIZE_BITS: usize = 11;
pub const SIZE: usize = 1 << SIZE_BITS;

pub fn new_terrain_random() -> Vec<u32> {
    let perlin = noise::Fbm::default();
    (0..SIZE * SIZE)
        .map(|i| {
            let loc = vec2(i % SIZE, i / SIZE).cast().unwrap() / SIZE as f64 * 2.0;
            let alt = perlin.get([loc.x, loc.y]) * 160.0 + 128.0;
            let alt_i = alt.min(255.0).max(0.0) as u32;
            alt_i * 0x1010101
        }).collect()
}
