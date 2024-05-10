pub const BOTTOM_NOTE: u8 = 21;
pub const NUM_KEYS: usize = 88;

pub const SAMPLE_RATE_hz: f32 = 44_100.0;
pub const AUDIO_BUS_LENGTH: usize = 16;

pub const BUS_OUT: usize = 0; // this is genuinely special, because this is what we connect to output
pub const BUS_DRY: usize = 1; // XXX this should be merely conventional and should eventually be deleted
