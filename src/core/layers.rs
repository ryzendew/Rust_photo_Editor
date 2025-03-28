use std::collections::HashMap;
use uuid::Uuid;
use image::{DynamicImage, RgbaImage, GenericImageView, Rgba};
use crate::vector::VectorShape;

/// Represents a layer type in the document
#[derive(Debug, Clone, PartialEq)]
pub enum LayerType {
    Pixel,             // Regular raster layer
    Vector,            // Vector graphics
    Text,              // Text layer
    Adjustment,        // Non-destructive adjustment
    LiveFilter,        // Non-destructive filter
    Group,             // Layer group
    SmartObject,       // Linked or embedded smart object
    RAW,               // RAW development layer
}

/// Represents a single layer in the document
#[derive(Debug, Clone)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: LayerBlendMode,
    pub mask: Option<LayerMask>,
    pub children: Vec<Layer>,
    
    // Type-specific data
    pub pixel_data: Option<DynamicImage>,
    pub vector_data: Option<Vec<VectorShape>>,
    pub adjustment_data: Option<Box<dyn AdjustmentLayer>>,
    pub filter_data: Option<Box<dyn FilterLayer>>,
    pub text_data: Option<TextLayerData>,
    pub smart_object_path: Option<String>,
    
    // Position and transform
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

/// Blending modes for layers
#[derive(Debug, Clone, PartialEq)]
pub enum LayerBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    // Advanced modes
    LinearDodge,
    LinearBurn,
    LinearLight,
    VividLight,
    PinLight,
    HardMix,
    Divide,
    Subtract,
    Negation,
}

impl Default for LayerBlendMode {
    fn default() -> Self {
        LayerBlendMode::Normal
    }
}

impl std::fmt::Display for LayerBlendMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerBlendMode::Normal => write!(f, "Normal"),
            LayerBlendMode::Multiply => write!(f, "Multiply"),
            LayerBlendMode::Screen => write!(f, "Screen"),
            LayerBlendMode::Overlay => write!(f, "Overlay"),
            LayerBlendMode::Darken => write!(f, "Darken"),
            LayerBlendMode::Lighten => write!(f, "Lighten"),
            LayerBlendMode::ColorDodge => write!(f, "Color Dodge"),
            LayerBlendMode::ColorBurn => write!(f, "Color Burn"),
            LayerBlendMode::HardLight => write!(f, "Hard Light"),
            LayerBlendMode::SoftLight => write!(f, "Soft Light"),
            LayerBlendMode::Difference => write!(f, "Difference"),
            LayerBlendMode::Exclusion => write!(f, "Exclusion"),
            LayerBlendMode::Hue => write!(f, "Hue"),
            LayerBlendMode::Saturation => write!(f, "Saturation"),
            LayerBlendMode::Color => write!(f, "Color"),
            LayerBlendMode::Luminosity => write!(f, "Luminosity"),
            LayerBlendMode::LinearDodge => write!(f, "Linear Dodge"),
            LayerBlendMode::LinearBurn => write!(f, "Linear Burn"),
            LayerBlendMode::LinearLight => write!(f, "Linear Light"),
            LayerBlendMode::VividLight => write!(f, "Vivid Light"),
            LayerBlendMode::PinLight => write!(f, "Pin Light"),
            LayerBlendMode::HardMix => write!(f, "Hard Mix"),
            LayerBlendMode::Divide => write!(f, "Divide"),
            LayerBlendMode::Subtract => write!(f, "Subtract"),
            LayerBlendMode::Negation => write!(f, "Negation"),
        }
    }
}

/// Manages all layers in the document
#[derive(Debug, Clone)]
pub struct LayerManager {
    pub layers: Vec<Layer>,
    pub active_layer_id: Option<String>,
}

impl LayerManager {
    /// Create a new layer manager
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            active_layer_id: None,
        }
    }
    
    /// Add a layer to the top of the stack
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer.clone());
        
        // Set as active if first layer
        if self.active_layer_id.is_none() {
            self.active_layer_id = Some(layer.id);
        }
    }
    
    /// Remove a layer by index
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index < self.layers.len() {
            // Update active layer index if needed
            if let Some(active_index) = self.active_layer_id.as_ref().map(|id| self.layers.iter().position(|l| l.id == id)) {
                if active_index == Some(index) {
                    // If we're removing the active layer, set the next one as active
                    self.active_layer_id = if self.layers.len() > 1 {
                        Some(self.layers[if index > 0 { index - 1 } else { 0 }].id.clone())
                    } else {
                        None
                    };
                } else if active_index > Some(index) {
                    // If the active layer is above the removed one, adjust its index
                    self.active_layer_id = Some(self.layers[if index > 0 { index - 1 } else { 0 }].id.clone());
                }
            }
            
            Some(self.layers.remove(index))
        } else {
            None
        }
    }
    
    /// Get a reference to all layers
    pub fn get_layers(&self) -> &[Layer] {
        &self.layers
    }
    
    /// Get a mutable reference to all layers
    pub fn get_layers_mut(&mut self) -> &mut Vec<Layer> {
        &mut self.layers
    }
    
    /// Get a reference to a layer by index
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }
    
    /// Get a mutable reference to a layer by index
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
    
    /// Get the active layer index
    pub fn get_active_layer_index(&self) -> Option<usize> {
        self.active_layer_id.as_ref().map(|id| self.layers.iter().position(|l| l.id == id).unwrap())
    }
    
    /// Set the active layer by index
    pub fn set_active_layer(&mut self, index: usize) -> Result<(), &'static str> {
        if index < self.layers.len() {
            self.active_layer_id = Some(self.layers[index].id.clone());
            Ok(())
        } else {
            Err("Layer index out of bounds")
        }
    }
    
    /// Get a reference to the active layer
    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.active_layer_id.as_ref().and_then(|id| self.layers.iter().find(|l| l.id == id))
    }
    
    /// Get a mutable reference to the active layer
    pub fn get_active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active_layer_id.as_ref().map(|id| self.layers.iter_mut().find(|l| l.id == id).unwrap())
    }
    
    /// Move a layer up in the stack
    pub fn move_layer_up(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.layers.len() {
            return Err("Layer index out of bounds");
        }
        
        if index == self.layers.len() - 1 {
            return Err("Layer already at the top");
        }
        
        self.layers.swap(index, index + 1);
        
        // Update active layer index if needed
        if let Some(active_index) = self.active_layer_id.as_ref().map(|id| self.layers.iter().position(|l| l.id == id)) {
            if active_index == Some(index) {
                self.active_layer_id = Some(self.layers[index + 1].id.clone());
            } else if active_index == Some(index + 1) {
                self.active_layer_id = Some(self.layers[index].id.clone());
            }
        }
        
        Ok(())
    }
    
    /// Move a layer down in the stack
    pub fn move_layer_down(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.layers.len() {
            return Err("Layer index out of bounds");
        }
        
        if index == 0 {
            return Err("Layer already at the bottom");
        }
        
        self.layers.swap(index, index - 1);
        
        // Update active layer index if needed
        if let Some(active_index) = self.active_layer_id.as_ref().map(|id| self.layers.iter().position(|l| l.id == id)) {
            if active_index == Some(index) {
                self.active_layer_id = Some(self.layers[index - 1].id.clone());
            } else if active_index == Some(index - 1) {
                self.active_layer_id = Some(self.layers[index].id.clone());
            }
        }
        
        Ok(())
    }
    
    /// Flatten all layers to a single image
    pub fn flatten_to_image(&self, width: u32, height: u32) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        // Create a blank image
        let mut result = RgbaImage::new(width, height);
        
        // Fill with white
        for pixel in result.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }
        
        let result_img = DynamicImage::ImageRgba8(result);
        
        // TODO: Implement actual layer compositing with blend modes and filters
        
        Ok(result_img)
    }
}

/// Text layer data
#[derive(Debug, Clone)]
pub struct TextLayerData {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: [u8; 4],
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub alignment: TextAlignment,
    pub letter_spacing: f32,
    pub line_height: f32,
}

/// Text alignment
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// Adjustment layer trait
pub trait AdjustmentLayer: std::fmt::Debug + Send + Sync {
    fn apply(&self, image: &DynamicImage) -> DynamicImage;
    fn get_type(&self) -> AdjustmentType;
    fn clone_box(&self) -> Box<dyn AdjustmentLayer>;
}

impl Clone for Box<dyn AdjustmentLayer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Adjustment type
#[derive(Debug, Clone, PartialEq)]
pub enum AdjustmentType {
    HSL,
    Curves,
    Levels,
    BlackAndWhite,
    ColorBalance,
    Vibrance,
    Brightness,
    Exposure,
    Gradient,
    Invert,
    Threshold,
    Posterize,
    SelectiveColor,
    ChannelMixer,
}

/// Filter layer trait
pub trait FilterLayer: std::fmt::Debug + Send + Sync {
    fn apply(&self, image: &DynamicImage) -> DynamicImage;
    fn get_type(&self) -> FilterType;
    fn clone_box(&self) -> Box<dyn FilterLayer>;
}

impl Clone for Box<dyn FilterLayer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Filter type
#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    GaussianBlur,
    MotionBlur,
    Sharpen,
    UnsharpMask,
    Noise,
    Distortion,
    Lighting,
    Stylize,
    Custom(String),
}

/// Layer mask
#[derive(Debug, Clone)]
pub struct LayerMask {
    pub id: String,
    pub bitmap: RgbaImage,
    pub enabled: bool,
    pub linked: bool,
}

impl LayerMask {
    pub fn new(width: u32, height: u32) -> Self {
        let id = Uuid::new_v4().to_string();
        let bitmap = RgbaImage::new(width, height);
        
        Self {
            id,
            bitmap,
            enabled: true,
            linked: true,
        }
    }
    
    pub fn from_image(image: RgbaImage) -> Self {
        let id = Uuid::new_v4().to_string();
        
        Self {
            id,
            bitmap: image,
            enabled: true,
            linked: true,
        }
    }
}

impl Layer {
    pub fn new(name: String, layer_type: LayerType, width: u32, height: u32) -> Self {
        let id = Uuid::new_v4().to_string();
        
        Self {
            id,
            name,
            layer_type,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: LayerBlendMode::default(),
            mask: None,
            children: Vec::new(),
            
            pixel_data: None,
            vector_data: None,
            adjustment_data: None,
            filter_data: None,
            text_data: None,
            smart_object_path: None,
            
            x: 0,
            y: 0,
            width,
            height,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
    
    pub fn from_image(name: String, image: DynamicImage) -> Self {
        let (width, height) = image.dimensions();
        let mut layer = Self::new(name, LayerType::Pixel, width, height);
        layer.pixel_data = Some(image);
        layer
    }
    
    pub fn create_vector_layer(name: String, width: u32, height: u32) -> Self {
        let mut layer = Self::new(name, LayerType::Vector, width, height);
        layer.vector_data = Some(Vec::new());
        layer
    }
    
    pub fn create_text_layer(name: String, width: u32, height: u32, text: String) -> Self {
        let mut layer = Self::new(name, LayerType::Text, width, height);
        layer.text_data = Some(TextLayerData {
            text,
            font_family: "Arial".to_string(),
            font_size: 12.0,
            color: [0, 0, 0, 255],
            bold: false,
            italic: false,
            underline: false,
            alignment: TextAlignment::Left,
            letter_spacing: 0.0,
            line_height: 1.2,
        });
        layer
    }
    
    pub fn create_adjustment_layer(name: String, width: u32, height: u32, adjustment_type: AdjustmentType) -> Self {
        let mut layer = Self::new(name, LayerType::Adjustment, width, height);
        
        let adjustment: Box<dyn AdjustmentLayer> = match adjustment_type {
            AdjustmentType::HSL => Box::new(HSLAdjustment::default()),
            AdjustmentType::Curves => Box::new(CurvesAdjustment::default()),
            AdjustmentType::Levels => Box::new(LevelsAdjustment::default()),
            AdjustmentType::BlackAndWhite => Box::new(BlackAndWhiteAdjustment::default()),
            AdjustmentType::ColorBalance => Box::new(ColorBalanceAdjustment::default()),
            // Add implementations for other adjustment types
            _ => Box::new(HSLAdjustment::default()), // Default for now
        };
        
        layer.adjustment_data = Some(adjustment);
        layer
    }
    
    pub fn create_filter_layer(name: String, width: u32, height: u32, filter_type: FilterType) -> Self {
        let mut layer = Self::new(name, LayerType::LiveFilter, width, height);
        
        let filter: Box<dyn FilterLayer> = match filter_type {
            FilterType::GaussianBlur => Box::new(GaussianBlurFilter::default()),
            FilterType::Sharpen => Box::new(SharpenFilter::default()),
            // Add implementations for other filter types
            _ => Box::new(GaussianBlurFilter::default()), // Default for now
        };
        
        layer.filter_data = Some(filter);
        layer
    }
    
    pub fn create_group_layer(name: String, width: u32, height: u32) -> Self {
        Self::new(name, LayerType::Group, width, height)
    }
    
    pub fn add_child(&mut self, child: Layer) {
        if self.layer_type == LayerType::Group {
            self.children.push(child);
        }
    }
    
    pub fn add_mask(&mut self, mask: RgbaImage) {
        self.mask = Some(LayerMask::from_image(mask));
    }
    
    pub fn create_empty_mask(&mut self) {
        self.mask = Some(LayerMask::new(self.width, self.height));
    }
    
    pub fn render(&self) -> Option<DynamicImage> {
        match self.layer_type {
            LayerType::Pixel => self.pixel_data.clone(),
            LayerType::Vector => {
                // This would render vector shapes to raster
                // For now, just return a placeholder
                None
            },
            LayerType::Text => {
                // This would render text to raster
                // For now, just return a placeholder
                None
            },
            LayerType::Adjustment => {
                // Apply adjustment to layers below
                None
            },
            LayerType::LiveFilter => {
                // Apply filter to layers below
                None
            },
            LayerType::Group => {
                // Composite children
                None
            },
            LayerType::SmartObject => {
                // Render smart object
                None
            },
            LayerType::RAW => {
                // Render RAW with current development settings
                None
            },
        }
    }
}

// HSL Adjustment
#[derive(Debug, Clone)]
pub struct HSLAdjustment {
    pub hue: f32,       // -180 to 180
    pub saturation: f32, // -100 to 100
    pub lightness: f32,  // -100 to 100
    pub colorize: bool,
    pub color_hue: f32,     // 0 to 360
    pub color_saturation: f32, // 0 to 100
    pub ranges: HSLRanges,  // Range adjustments for specific colors
}

#[derive(Debug, Clone)]
pub struct HSLRanges {
    pub reds: (f32, f32, f32),     // (hue, saturation, lightness)
    pub yellows: (f32, f32, f32),
    pub greens: (f32, f32, f32),
    pub cyans: (f32, f32, f32),
    pub blues: (f32, f32, f32),
    pub magentas: (f32, f32, f32),
}

impl Default for HSLRanges {
    fn default() -> Self {
        Self {
            reds: (0.0, 0.0, 0.0),
            yellows: (0.0, 0.0, 0.0),
            greens: (0.0, 0.0, 0.0),
            cyans: (0.0, 0.0, 0.0),
            blues: (0.0, 0.0, 0.0),
            magentas: (0.0, 0.0, 0.0),
        }
    }
}

impl Default for HSLAdjustment {
    fn default() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
            colorize: false,
            color_hue: 0.0,
            color_saturation: 0.0,
            ranges: HSLRanges::default(),
        }
    }
}

impl AdjustmentLayer for HSLAdjustment {
    fn apply(&self, image: &DynamicImage) -> DynamicImage {
        let mut output = image.clone();
        let (width, height) = output.dimensions();
        
        // Apply HSL adjustment to each pixel
        for y in 0..height {
            for x in 0..width {
                let pixel = output.get_pixel(x, y);
                let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
                
                // Convert RGB to HSL
                let (mut h, mut s, mut l) = rgb_to_hsl(r, g, b);
                
                if self.colorize {
                    // Override hue and saturation, adjust lightness
                    h = self.color_hue / 360.0;
                    s = self.color_saturation / 100.0;
                    l = adjust_value(l, self.lightness / 100.0);
                } else {
                    // Apply global adjustments
                    h = (h + self.hue / 360.0) % 1.0;
                    s = adjust_value(s, self.saturation / 100.0);
                    l = adjust_value(l, self.lightness / 100.0);
                    
                    // Apply range-specific adjustments
                    // This would check which color range the pixel falls into
                    // and apply the specific adjustments for that range
                }
                
                // Convert back to RGB
                let (r2, g2, b2) = hsl_to_rgb(h, s, l);
                
                output.put_pixel(x, y, image::Rgba([r2, g2, b2, a]));
            }
        }
        
        output
    }
    
    fn get_type(&self) -> AdjustmentType {
        AdjustmentType::HSL
    }
    
    fn clone_box(&self) -> Box<dyn AdjustmentLayer> {
        Box::new(self.clone())
    }
}

// Helper functions for HSL conversion
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    
    let mut h = 0.0;
    let mut s = 0.0;
    let l = (max + min) / 2.0;
    
    if max != min {
        let d = max - min;
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        
        h = if max == r {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        
        h /= 6.0;
    }
    
    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        
        p
    };
    
    if s == 0.0 {
        // Achromatic (gray)
        let gray = (l * 255.0) as u8;
        return (gray, gray, gray);
    }
    
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

// Helper function to adjust a value in the range [0, 1] by a delta in the range [-1, 1]
fn adjust_value(value: f32, delta: f32) -> f32 {
    if delta > 0.0 {
        value + (1.0 - value) * delta
    } else {
        value + value * delta
    }
}

// Curves Adjustment with more functionality
#[derive(Debug, Clone)]
pub struct CurvesAdjustment {
    pub curves: HashMap<char, Vec<(f32, f32)>>, // Channel -> points
    pub rgb_composite: bool,                    // Apply RGB curve to all channels
}

impl Default for CurvesAdjustment {
    fn default() -> Self {
        let mut curves = HashMap::new();
        curves.insert('r', vec![(0.0, 0.0), (1.0, 1.0)]);
        curves.insert('g', vec![(0.0, 0.0), (1.0, 1.0)]);
        curves.insert('b', vec![(0.0, 0.0), (1.0, 1.0)]);
        curves.insert('a', vec![(0.0, 0.0), (1.0, 1.0)]);
        curves.insert('c', vec![(0.0, 0.0), (1.0, 1.0)]); // Composite curve for RGB
        Self { 
            curves,
            rgb_composite: false,
        }
    }
}

impl AdjustmentLayer for CurvesAdjustment {
    fn apply(&self, image: &DynamicImage) -> DynamicImage {
        let mut output = image.clone();
        let (width, height) = output.dimensions();
        
        // Precompute lookup tables for each channel
        let mut luts = HashMap::new();
        for (channel, points) in &self.curves {
            let mut lut = [0u8; 256];
            
            // Generate lookup table for this curve
            for i in 0..256 {
                let x = i as f32 / 255.0;
                let y = interpolate_curve(points, x);
                lut[i] = (y.clamp(0.0, 1.0) * 255.0) as u8;
            }
            
            luts.insert(*channel, lut);
        }
        
        // Apply curves to each pixel
        for y in 0..height {
            for x in 0..width {
                let pixel = output.get_pixel(x, y);
                let mut r = pixel[0];
                let mut g = pixel[1];
                let mut b = pixel[2];
                let a = pixel[3];
                
                // Apply composite curve if enabled
                if self.rgb_composite && luts.contains_key(&'c') {
                    let lut = luts.get(&'c').unwrap();
                    r = lut[r as usize];
                    g = lut[g as usize];
                    b = lut[b as usize];
                }
                
                // Apply individual channel curves
                if luts.contains_key(&'r') {
                    let lut = luts.get(&'r').unwrap();
                    r = lut[r as usize];
                }
                
                if luts.contains_key(&'g') {
                    let lut = luts.get(&'g').unwrap();
                    g = lut[g as usize];
                }
                
                if luts.contains_key(&'b') {
                    let lut = luts.get(&'b').unwrap();
                    b = lut[b as usize];
                }
                
                output.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }
        }
        
        output
    }
    
    fn get_type(&self) -> AdjustmentType {
        AdjustmentType::Curves
    }
    
    fn clone_box(&self) -> Box<dyn AdjustmentLayer> {
        Box::new(self.clone())
    }
}

// Helper function to interpolate a point on a curve
fn interpolate_curve(points: &[(f32, f32)], x: f32) -> f32 {
    if points.len() < 2 {
        return x; // No curve, return identity
    }
    
    // Find the segment that contains x
    let mut left = &points[0];
    let mut right = &points[1];
    
    for i in 1..points.len() {
        if points[i].0 >= x {
            right = &points[i];
            left = &points[i-1];
            break;
        }
    }
    
    // Interpolate
    if right.0 == left.0 {
        return left.1;
    }
    
    let t = (x - left.0) / (right.0 - left.0);
    left.1 + t * (right.1 - left.1)
} 