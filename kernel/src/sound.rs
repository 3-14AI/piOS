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


    pub struct VoiceAssistant {
        pub is_listening: bool,
    }

    impl VoiceAssistant {
        pub fn new() -> (v: Self)
            ensures v.is_listening == false
        {
            VoiceAssistant { is_listening: false }
        }

        pub fn start_listening(&mut self)
            ensures self.is_listening == true
        {
            self.is_listening = true;
        }

        pub fn stop_listening(&mut self)
            ensures self.is_listening == false
        {
            self.is_listening = false;
        }

        pub fn process_audio_buffer(&mut self, buffer: &AudioBuffer) -> (res: bool)
            ensures
                res == (self.is_listening && buffer.capacity > 0)
        {
            self.is_listening && buffer.capacity > 0
        }

        // Verus mock for fetching command. We return a boolean indicating success.
        pub fn get_recognized_command_id(&self) -> (res: u32)
            ensures
                self.is_listening ==> res == 1,
                !self.is_listening ==> res == 0
        {
            if self.is_listening { 1 } else { 0 }
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
        pub is_recognition_loop_running: bool,
    }

    impl HdaSoundDriver {
        pub fn new() -> (d: Self)
            ensures
                d.initialized == true,
                d.is_recognition_loop_running == false
        {
            HdaSoundDriver {
                initialized: true,
                is_recognition_loop_running: false,
            }
        }

        pub fn start_voice_recognition_loop(&mut self)
            ensures self.is_recognition_loop_running == true
        {
            self.is_recognition_loop_running = true;
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
pub struct VoiceAssistant {
    pub is_listening: bool,
}

#[cfg(not(feature = "verus"))]
impl VoiceAssistant {
    pub fn new() -> Self {
        VoiceAssistant {
            is_listening: false,
        }
    }

    pub fn start_listening(&mut self) {
        self.is_listening = true;
    }

    pub fn stop_listening(&mut self) {
        self.is_listening = false;
    }

    pub fn process_audio_buffer(&mut self, buffer: &AudioBuffer) -> bool {
        self.is_listening && buffer.capacity > 0
    }

    pub fn get_recognized_command_id(&self) -> u32 {
        if self.is_listening {
            1
        } else {
            0
        }
    }

    pub fn get_recognized_command(&self) -> Option<alloc::string::String> {
        if self.is_listening {
            Some(alloc::string::String::from("recognized voice command"))
        } else {
            None
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
impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
pub struct HdaSoundDriver {
    pub initialized: bool,
    pub is_recognition_loop_running: bool,
    pub assistant: VoiceAssistant,
}

#[cfg(not(feature = "verus"))]
impl HdaSoundDriver {
    pub fn new() -> Self {
        HdaSoundDriver {
            initialized: true,
            is_recognition_loop_running: false,
            assistant: VoiceAssistant::new(),
        }
    }

    pub fn start_voice_recognition_loop(&mut self) {
        self.is_recognition_loop_running = true;
        self.assistant.start_listening();
    }

    // Simulate one iteration of the continuous loop
    pub fn poll_recognition_loop(&mut self, buffer: &AudioBuffer) -> Option<alloc::string::String> {
        if self.is_recognition_loop_running {
            if self.assistant.process_audio_buffer(buffer) {
                return self.assistant.get_recognized_command();
            }
        }
        None
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
        let mut drv = HdaSoundDriver::new();
        assert!(drv.initialized);
        assert!(!drv.is_recognition_loop_running);

        drv.start_voice_recognition_loop();
        assert!(drv.is_recognition_loop_running);

        let buf = AudioBuffer::new(1024);
        let cmd = drv.poll_recognition_loop(&buf);
        assert_eq!(
            cmd,
            Some(alloc::string::String::from("recognized voice command"))
        );

        let drv_def = HdaSoundDriver::default();
        assert!(drv_def.initialized);
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
        assert!(!stream.state.is_playing);
    }

    #[test]
    fn test_voice_assistant() {
        let mut assistant = VoiceAssistant::new();
        assert!(!assistant.is_listening);

        assistant.start_listening();
        assert!(assistant.is_listening);

        let buf = AudioBuffer::new(1024);
        let res = assistant.process_audio_buffer(&buf);
        assert!(res);
        assert_eq!(
            assistant.get_recognized_command(),
            Some(alloc::string::String::from("recognized voice command"))
        );

        assistant.stop_listening();
        assert!(!assistant.is_listening);
        let res_stopped = assistant.process_audio_buffer(&buf);
        assert!(!res_stopped);
        assert_eq!(assistant.get_recognized_command(), None);
    }

    #[test]
    fn test_audio_mixer() {
        let mixer = AudioMixer::new();
        assert_eq!(mixer.master_volume, 100);
    }
}
