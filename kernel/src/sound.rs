#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct PcmState {
        pub is_playing: bool,
    }

    pub struct AudioBuffer {
        pub capacity: usize,
        pub head: usize,
        pub tail: usize,
    }

    impl AudioBuffer {
        pub fn new(capacity: usize) -> (b: Self)
            requires capacity > 0
            ensures
                b.capacity == capacity,
                b.head == 0,
                b.tail == 0
        {
            AudioBuffer {
                capacity,
                head: 0,
                tail: 0,
            }
        }
    }

    pub struct PcmStream {
        pub sample_rate: u32,
        pub channels: u8,
        pub state: PcmState,
    }

    impl PcmStream {
        pub fn new(sample_rate: u32, channels: u8) -> (s: Self)
            ensures
                s.sample_rate == sample_rate,
                s.channels == channels,
                s.state.is_playing == false
        {
            PcmStream {
                sample_rate,
                channels,
                state: PcmState { is_playing: false },
            }
        }
    }

    pub struct AudioMixer {
        pub master_volume: u8,
    }

    impl AudioMixer {
        pub fn new() -> (m: Self)
            ensures m.master_volume == 100
        {
            AudioMixer { master_volume: 100 }
        }
    }

    pub struct HdaSoundDriver {
        pub initialized: bool,
    }

    impl HdaSoundDriver {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            HdaSoundDriver { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct PcmState {
    pub is_playing: bool,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct AudioBuffer {
    pub capacity: usize,
    pub head: usize,
    pub tail: usize,
}

#[cfg(not(feature = "verus"))]
impl AudioBuffer {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        AudioBuffer {
            capacity,
            head: 0,
            tail: 0,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct PcmStream {
    pub sample_rate: u32,
    pub channels: u8,
    pub state: PcmState,
}

#[cfg(not(feature = "verus"))]
impl PcmStream {
    pub fn new(sample_rate: u32, channels: u8) -> Self {
        PcmStream {
            sample_rate,
            channels,
            state: PcmState { is_playing: false },
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct AudioMixer {
    pub master_volume: u8,
}

#[cfg(not(feature = "verus"))]
impl AudioMixer {
    pub fn new() -> Self {
        AudioMixer { master_volume: 100 }
    }
}

#[cfg(not(feature = "verus"))]
pub struct HdaSoundDriver {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl HdaSoundDriver {
    pub fn new() -> Self {
        HdaSoundDriver { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for HdaSoundDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_driver() {
        let drv = HdaSoundDriver::new();
        assert_eq!(drv.initialized, true);
        let drv_def = HdaSoundDriver::default();
        assert_eq!(drv_def.initialized, true);
    }

    #[test]
    fn test_audio_buffer() {
        let buf = AudioBuffer::new(1024);
        assert_eq!(buf.capacity, 1024);
        assert_eq!(buf.head, 0);
        assert_eq!(buf.tail, 0);
    }

    #[test]
    fn test_pcm_stream() {
        let stream = PcmStream::new(44100, 2);
        assert_eq!(stream.sample_rate, 44100);
        assert_eq!(stream.channels, 2);
        assert_eq!(stream.state.is_playing, false);
    }

    #[test]
    fn test_audio_mixer() {
        let mixer = AudioMixer::new();
        assert_eq!(mixer.master_volume, 100);
    }
}
