use crate::algorithms::perlin_noise::perlin_noise;

/// Return the humidity level in [0; 1] at this location
pub fn humidity(x: f64, y: f64) -> f64 {
    perlin_noise(x, y, 0.0005, 0.5) + 0.5
}
