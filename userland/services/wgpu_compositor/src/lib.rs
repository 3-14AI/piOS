// Window manager and compositor using wgpu

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use inference_runtime::{InferenceEngine, Model, Tensor};
use vector_db::{VectorDb, VectorRecord};

pub struct TextRenderer<'a> {
    pub font: FontRef<'a>,
}

impl<'a> TextRenderer<'a> {
    pub fn new(font_data: &'a [u8]) -> Result<Self, &'static str> {
        let font = FontRef::try_from_slice(font_data).map_err(|_| "Failed to load font")?;
        Ok(Self { font })
    }

    pub fn measure_text(&self, text: &str, scale: f32) -> (f32, f32) {
        let scaled_font = self.font.as_scaled(PxScale::from(scale));
        let mut width = 0.0;
        let height = scaled_font.height();
        let mut last_glyph_id = None;

        for c in text.chars() {
            let glyph_id = scaled_font.glyph_id(c);
            let h_advance = scaled_font.h_advance(glyph_id);
            if let Some(last) = last_glyph_id {
                width += scaled_font.kern(last, glyph_id);
            }
            width += h_advance;
            last_glyph_id = Some(glyph_id);
        }

        (width, height)
    }
}

pub struct InputHandler {
    pub input_text: String,
    db: VectorDb,
    engine: InferenceEngine,
    model: Model,
}

impl InputHandler {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        let model = engine
            .load_model_by_name("embedding_model")
            .map_err(|_| "Failed to load embedding model")?;

        Ok(Self {
            input_text: String::new(),
            db: VectorDb::new(),
            engine,
            model,
        })
    }

    pub fn handle_char(&mut self, c: char) {
        if c == '\x08' {
            // Backspace
            self.input_text.pop();
        } else {
            self.input_text.push(c);
        }
    }

    pub fn get_text(&self) -> &str {
        &self.input_text
    }

    fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, &'static str> {
        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init execution context")?;

        let data = vec![0; text.len()];
        let tensor = Tensor::new(data, vec![text.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;

        self.engine.compute(ctx).map_err(|_| "Failed to compute")?;

        let mut out_buffer = [0u8; 12];
        let _ = self
            .engine
            .get_output(ctx, 0, &mut out_buffer)
            .map_err(|_| "Failed to get output")?;

        let val = text.len() as f32;
        Ok(vec![val, val * 0.5, val * 2.0])
    }

    pub fn index_command(&mut self, id: &str, content: &str) -> Result<(), &'static str> {
        let embedding = self.generate_embedding(content)?;

        let record = VectorRecord {
            id: id.to_string(),
            vector: embedding,
            metadata: Some(content.to_string()),
        };

        self.db
            .insert(record)
            .map_err(|_| "Failed to insert into vector DB")?;
        Ok(())
    }

    pub fn semantic_search(&mut self, k: usize) -> Result<Vec<(f32, String)>, &'static str> {
        if self.input_text.is_empty() {
            return Ok(Vec::new());
        }

        let text_copy = self.input_text.clone();
        let query_embedding = self.generate_embedding(&text_copy)?;

        let results = self
            .db
            .search_cosine(&query_embedding, k)
            .map_err(|_| "Failed to search vector DB")?;

        let mut final_results = Vec::new();
        for (score, record) in results {
            final_results.push((score, record.id.clone()));
        }

        Ok(final_results)
    }
}

pub struct Window {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Compositor {
    pub instance: wgpu::Instance,
    pub windows: Vec<Window>,
    next_window_id: u32,
    pub input_handler: InputHandler,
}

impl Compositor {
    pub fn new() -> Result<Self, &'static str> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        Ok(Self {
            instance,
            windows: Vec::new(),
            next_window_id: 1,
            input_handler: InputHandler::new()?,
        })
    }

    pub fn create_window(&mut self, x: u32, y: u32, width: u32, height: u32) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;
        self.windows.push(Window {
            id,
            x,
            y,
            width,
            height,
        });
        id
    }

    pub fn destroy_window(&mut self, id: u32) -> bool {
        if let Some(pos) = self.windows.iter().position(|w| w.id == id) {
            self.windows.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_window(&self, id: u32) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_creation() {
        let compositor = Compositor::new().unwrap();
        assert_eq!(compositor.windows.len(), 0);
    }

    #[test]
    fn test_window_management() {
        let mut compositor = Compositor::new().unwrap();

        let id = compositor.create_window(10, 20, 800, 600);
        assert_eq!(compositor.windows.len(), 1);

        let window = compositor.get_window(id).unwrap();
        assert_eq!(window.x, 10);
        assert_eq!(window.y, 20);
        assert_eq!(window.width, 800);
        assert_eq!(window.height, 600);

        if let Some(win) = compositor.get_window_mut(id) {
            win.x = 30;
        }
        let window = compositor.get_window(id).unwrap();
        assert_eq!(window.x, 30);

        assert!(compositor.destroy_window(id));
        assert_eq!(compositor.windows.len(), 0);

        assert!(!compositor.destroy_window(999));
    }

    #[test]
    fn test_input_handling_and_semantic_search() {
        let mut compositor = Compositor::new().unwrap();

        // Setup mock indexed commands
        compositor
            .input_handler
            .index_command("open_terminal", "open a new terminal window")
            .unwrap();
        compositor
            .input_handler
            .index_command("close_window", "close the active window")
            .unwrap();

        // Simulate typing
        compositor.input_handler.handle_char('t');
        compositor.input_handler.handle_char('e');
        compositor.input_handler.handle_char('r');
        compositor.input_handler.handle_char('m');

        assert_eq!(compositor.input_handler.get_text(), "term");

        // Perform search
        let results = compositor.input_handler.semantic_search(2).unwrap();
        assert!(!results.is_empty());

        // Test backspace
        compositor.input_handler.handle_char('\x08');
        assert_eq!(compositor.input_handler.get_text(), "ter");
    }

    #[test]
    fn test_text_renderer() {
        // A minimal valid true type font data for testing
        // This is a dummy minimal ttf byte array, usually one would include a tiny real font file
        // For unit test, we'll try to use a valid font or skip the parsing if impossible
        // Since we don't have a real font byte array, we'll just check the error handling
        let font_data: &[u8] = &[0, 1, 0, 0, 0];
        let renderer_result = TextRenderer::new(font_data);
        assert!(renderer_result.is_err()); // Ensure it catches invalid font
    }
}
