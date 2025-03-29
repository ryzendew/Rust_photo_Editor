use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use uuid::Uuid;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use crate::core::layer::{Layer, LayerManager};
use std::collections::HashMap;

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

impl fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorSpace::SRGB => write!(f, "sRGB"),
            ColorSpace::AdobeRGB => write!(f, "Adobe RGB"),
            ColorSpace::ProPhotoRGB => write!(f, "ProPhoto RGB"),
            ColorSpace::CMYK => write!(f, "CMYK"),
            ColorSpace::Grayscale => write!(f, "Grayscale"),
            ColorSpace::LAB => write!(f, "LAB"),
            ColorSpace::HSL => write!(f, "HSL"),
            ColorSpace::HDR => write!(f, "HDR"),
            ColorSpace::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitDepth {
    Bit8,
    Bit16,
    Bit32,
}

impl fmt::Display for BitDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitDepth::Bit8 => write!(f, "8-bit"),
            BitDepth::Bit16 => write!(f, "16-bit"),
            BitDepth::Bit32 => write!(f, "32-bit"),
        }
    }
}

/// Represents the format of a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    /// JPEG format
    JPEG,
    /// PNG format
    PNG,
    /// TIFF format
    TIFF,
    /// WebP format
    WebP,
    /// Affinity Photo document
    AffinityPhoto,
    /// Custom native format for this editor
    Native,
}

impl DocumentFormat {
    /// Convert a file extension to a format
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(DocumentFormat::JPEG),
            "png" => Some(DocumentFormat::PNG),
            "tif" | "tiff" => Some(DocumentFormat::TIFF),
            "webp" => Some(DocumentFormat::WebP),
            "afphoto" => Some(DocumentFormat::AffinityPhoto),
            "aprs" => Some(DocumentFormat::Native),
            _ => None,
        }
    }
    
    /// Convert a format to a file extension
    pub fn to_extension(&self) -> &'static str {
        match self {
            DocumentFormat::JPEG => "jpg",
            DocumentFormat::PNG => "png",
            DocumentFormat::TIFF => "tiff",
            DocumentFormat::WebP => "webp",
            DocumentFormat::AffinityPhoto => "afphoto",
            DocumentFormat::Native => "aprs",
        }
    }
}

/// Metadata for a document
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentMetadata {
    /// Title of the document
    pub title: String,
    /// Author of the document
    pub author: Option<String>,
    /// Description of the document
    pub description: Option<String>,
    /// Copyright information
    pub copyright: Option<String>,
    /// Keywords/tags
    pub keywords: Vec<String>,
    /// Creation time
    pub creation_time: SystemTime,
    /// Last modification time
    pub modification_time: SystemTime,
    /// Custom metadata
    pub custom: HashMap<String, String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            author: None,
            description: None,
            copyright: None,
            keywords: Vec::new(),
            creation_time: SystemTime::now(),
            modification_time: SystemTime::now(),
            custom: HashMap::new(),
        }
    }
}

/// Represents a document in the application
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    /// Path to the document file
    pub path: Option<PathBuf>,
    /// Format of the document
    pub format: DocumentFormat,
    /// Width of the document
    pub width: u32,
    /// Height of the document
    pub height: u32,
    /// Layers in the document
    pub layer_manager: LayerManager,
    /// Metadata for the document
    pub metadata: DocumentMetadata,
    /// DPI (dots per inch) for the document
    pub dpi: f32,
    /// Background color
    pub background_color: Rgba<u8>,
}

impl Document {
    /// Create a new document with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let mut layer_manager = LayerManager::new();
        
        // Add a background layer
        let background = Layer::new(width, height, "Background".to_string());
        layer_manager.add_layer(background);
        
        Self {
            path: None,
            format: DocumentFormat::Native,
            width,
            height,
            layer_manager,
            metadata: DocumentMetadata::default(),
            dpi: 300.0,
            background_color: Rgba([255, 255, 255, 255]), // White background
        }
    }
    
    /// Create a document from an existing image
    pub fn from_image(image: DynamicImage, path: Option<PathBuf>) -> Self {
        let width = image.width();
        let height = image.height();
        
        let mut layer_manager = LayerManager::new();
        
        // Convert to RGBA
        let rgba_image = image.to_rgba8();
        
        // Create a layer from the image
        let layer = Layer::from_image(rgba_image, "Background".to_string());
        layer_manager.add_layer(layer);
        
        // Determine format from path if available
        let format = if let Some(path) = &path {
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    DocumentFormat::from_extension(ext_str).unwrap_or(DocumentFormat::Native)
                } else {
                    DocumentFormat::Native
                }
            } else {
                DocumentFormat::Native
            }
        } else {
            DocumentFormat::Native
        };
        
        // Set title from filename if available
        let title = if let Some(path) = &path {
            if let Some(filename) = path.file_name() {
                if let Some(name) = filename.to_str() {
                    name.to_string()
                } else {
                    "Untitled".to_string()
                }
            } else {
                "Untitled".to_string()
            }
        } else {
            "Untitled".to_string()
        };
        
        let mut metadata = DocumentMetadata::default();
        metadata.title = title;
        
        Self {
            path,
            format,
            width,
            height,
            layer_manager,
            metadata,
            dpi: 300.0,
            background_color: Rgba([255, 255, 255, 255]), // White background
        }
    }
    
    /// Create a document from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        
        // Load the image
        let image = match image::open(path) {
            Ok(img) => img,
            Err(err) => return Err(format!("Failed to open image: {}", err)),
        };
        
        // Create a document from the image
        Ok(Self::from_image(image, Some(path.to_path_buf())))
    }
    
    /// Save the document to a file
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        
        // Determine format from path
        let format = if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                DocumentFormat::from_extension(ext_str).unwrap_or(self.format)
            } else {
                self.format
            }
        } else {
            self.format
        };
        
        // Flatten the image
        let flattened = self.layer_manager.flatten();
        
        // Convert to DynamicImage
        let dynamic_image = DynamicImage::ImageRgba8(flattened);
        
        // Save the image
        match format {
            DocumentFormat::JPEG => {
                if let Err(err) = dynamic_image.save_with_format(path, image::ImageFormat::Jpeg) {
                    return Err(format!("Failed to save as JPEG: {}", err));
                }
            }
            DocumentFormat::PNG => {
                if let Err(err) = dynamic_image.save_with_format(path, image::ImageFormat::Png) {
                    return Err(format!("Failed to save as PNG: {}", err));
                }
            }
            DocumentFormat::TIFF => {
                if let Err(err) = dynamic_image.save_with_format(path, image::ImageFormat::Tiff) {
                    return Err(format!("Failed to save as TIFF: {}", err));
                }
            }
            DocumentFormat::WebP => {
                if let Err(err) = dynamic_image.save_with_format(path, image::ImageFormat::WebP) {
                    return Err(format!("Failed to save as WebP: {}", err));
                }
            }
            DocumentFormat::AffinityPhoto | DocumentFormat::Native => {
                // Not implemented yet
                return Err("Saving in native format not yet implemented".to_string());
            }
        }
        
        // Update path and format
        self.path = Some(path.to_path_buf());
        self.format = format;
        
        // Update metadata
        self.metadata.modification_time = SystemTime::now();
        
        Ok(())
    }
    
    /// Resize the document
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.layer_manager.resize_all_layers(width, height);
    }
    
    /// Crop the document
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.layer_manager.crop_all_layers(x, y, width, height);
    }
    
    /// Add a new layer to the document
    pub fn add_layer(&mut self, layer: Layer) -> usize {
        self.layer_manager.add_layer(layer)
    }
    
    /// Create a new empty layer
    pub fn create_empty_layer(&mut self, name: String) -> usize {
        let layer = Layer::new(self.width, self.height, name);
        self.layer_manager.add_layer(layer)
    }
    
    /// Export the document as a DynamicImage
    pub fn export(&self) -> DynamicImage {
        let flattened = self.layer_manager.flatten();
        DynamicImage::ImageRgba8(flattened)
    }
    
    /// Open a document from a file path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        Self::from_file(path)
    }
} 