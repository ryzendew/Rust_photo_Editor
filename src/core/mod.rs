// Core module: Contains the main application logic and data structures

pub mod point;
pub mod layer;
pub mod selection;
pub mod canvas;
pub mod document;
pub mod history;
pub mod settings;

pub use point::Point;
pub use layer::{Layer, LayerManager, BlendMode};
pub use selection::Selection;
pub use canvas::Canvas;
pub use document::{Document, DocumentFormat, DocumentMetadata};
pub use history::{HistoryManager, HistoryCommand, HistoryState};
pub use settings::{Settings, PerformanceSettings, SaveSettings, DisplaySettings, SettingsManager};

use log::{debug, error, info, trace, warn};

/// Initialize logging for the application
pub fn init_logging() {
    // Initialize env_logger with a default configuration
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    info!("Logging initialized for Rust Photo");
    debug!("Debug logging enabled");
    trace!("Trace logging enabled");
}

/// Color representation in RGBA format
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color with RGBA values
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        trace!("Creating new color: rgba({}, {}, {}, {})", r, g, b, a);
        Self { r, g, b, a }
    }
    
    /// Create a new color with RGB values (alpha set to 1.0)
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        trace!("Creating new RGB color: rgb({}, {}, {})", r, g, b);
        Self { r, g, b, a: 1.0 }
    }
    
    /// Create a black color
    pub fn black() -> Self {
        trace!("Creating black color");
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
    
    /// Create a white color
    pub fn white() -> Self {
        trace!("Creating white color");
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
    
    /// Create a transparent color
    pub fn transparent() -> Self {
        trace!("Creating transparent color");
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

// Core application types
pub struct AppState {
    pub current_document: Option<Document>,
    pub documents: Vec<Document>,
    pub history_manager: HistoryManager,
    pub preferences: Preferences,
    pub clipboard: Option<ClipboardContent>,
}

pub enum ClipboardContent {
    Pixels(image::DynamicImage),
    Vectors(Vec<crate::vector::VectorShape>),
    Text(String),
    Layers(Vec<Layer>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    SRGB,
    AdobeRGB,
    ProPhotoRGB,
    CMYK,
    Grayscale,
    LAB,
    HSL,
    HDR,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitDepth {
    Bit8,
    Bit16,
    Bit32,
}

pub struct Preferences {
    pub default_color_space: ColorSpace,
    pub default_bit_depth: BitDepth,
    pub default_resolution: f64,
    pub auto_save_interval: u32, // seconds
    pub undo_limit: usize,
    pub use_gpu_acceleration: bool,
    pub thumbnail_size: (u32, u32),
}

impl Default for Preferences {
    fn default() -> Self {
        debug!("Creating default Preferences");
        Self {
            default_color_space: ColorSpace::SRGB,
            default_bit_depth: BitDepth::Bit8,
            default_resolution: 300.0,
            auto_save_interval: 300, // 5 minutes
            undo_limit: 100,
            use_gpu_acceleration: true,
            thumbnail_size: (256, 256),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        info!("Creating new AppState");
        Self {
            current_document: None,
            documents: Vec::new(),
            history_manager: HistoryManager::new(),
            preferences: Preferences::default(),
            clipboard: None,
        }
    }

    pub fn new_document(&mut self, width: u32, height: u32, color_space: ColorSpace, bit_depth: BitDepth) -> Document {
        info!("Creating new document: {}x{} with color space {:?} and bit depth {:?}", 
              width, height, color_space, bit_depth);
        let document = Document::new(width, height);
        self.documents.push(document.clone());
        self.current_document = Some(document.clone());
        debug!("Document created successfully");
        document
    }

    pub fn open_document(&mut self, path: &str) -> Result<Document, String> {
        info!("Opening document from path: {}", path);
        match Document::open(path) {
            Ok(document) => {
                info!("Document opened successfully");
                self.documents.push(document.clone());
                self.current_document = Some(document.clone());
                Ok(document)
            },
            Err(e) => {
                error!("Failed to open document from {}: {}", path, e);
                Err(e.to_string())
            },
        }
    }

    pub fn close_document(&mut self, document: &Document) {
        info!("Closing document");
        if let Some(index) = self.documents.iter().position(|d| d == document) {
            self.documents.remove(index);
            
            // Update current document if necessary
            if let Some(current) = &self.current_document {
                if std::ptr::eq(current, document) {
                    debug!("Closed document was the current document, updating current document");
                    self.current_document = self.documents.first().cloned();
                    if self.current_document.is_some() {
                        debug!("New current document set");
                    } else {
                        debug!("No documents remain open");
                    }
                }
            }
        } else {
            warn!("Attempted to close document that wasn't found");
        }
    }

    pub fn copy_selection(&mut self) {
        if let Some(document) = &self.current_document {
            info!("Copying selection from document");
            // Implementation will depend on selection type and content
            // This is a placeholder
        } else {
            warn!("Cannot copy selection - no document is open");
        }
    }

    pub fn paste(&mut self) {
        if let Some(document) = &mut self.current_document {
            if let Some(content) = &self.clipboard {
                info!("Pasting to document");
                match content {
                    ClipboardContent::Pixels(_) => debug!("Pasting pixel data"),
                    ClipboardContent::Vectors(_) => debug!("Pasting vector data"),
                    ClipboardContent::Text(text) => debug!("Pasting text: '{}'", text),
                    ClipboardContent::Layers(_) => debug!("Pasting layers"),
                }
            } else {
                warn!("Cannot paste - clipboard is empty");
            }
        } else {
            warn!("Cannot paste - no document is open");
        }
    }
} 