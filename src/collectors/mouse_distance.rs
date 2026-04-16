#![allow(dead_code)]
// Mouse distance tracking is integrated into the input collector (input.rs).
// The MouseMove events from rdev are handled there, with delta accumulation
// and periodic flushing every 30 seconds.
//
// Pixel → feet conversion:
//   Assume 96 DPI standard monitor.
//   1 inch = 96 pixels, 1 foot = 12 inches = 1152 pixels.
//   feet = total_delta_px / 1152.0

/// Pixels per foot at 96 DPI (standard Windows).
pub const PIXELS_PER_FOOT: f64 = 1152.0;

/// Convert pixel distance to feet.
pub fn pixels_to_feet(pixels: f64) -> f64 {
    pixels / PIXELS_PER_FOOT
}
