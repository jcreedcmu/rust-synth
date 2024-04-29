use anyhow::anyhow;

pub type JoinHandle = std::thread::JoinHandle<anyhow::Result<()>>;

pub fn freq_of_pitch(pitch: u8) -> f32 {
  440.0 * 2.0f32.powf(((pitch as f32) - 69.0) / 12.0)
}

// The PoisonException arising from grabbing a mutex contains some mutex data itself
// that doesn't implement Send, so replace it with a string
pub fn depoison<T, E>(res: Result<T, E>) -> anyhow::Result<T> {
  match res {
    Err(e) => Err(anyhow!("some kind of mutex poison exception was raised")),
    Ok(v) => Ok(v),
  }
}
