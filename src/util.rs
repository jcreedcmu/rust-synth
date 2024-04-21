use std::error::Error;

pub type Mostly<T> = Result<T, Box<dyn Error>>;

pub fn freq_of_pitch(pitch: u8) -> f32 {
  440.0 * 2.0f32.powf(((pitch as f32) - 69.0) / 12.0)
}
