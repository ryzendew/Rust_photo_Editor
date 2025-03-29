use image::{DynamicImage, ImageBuffer, Rgba};
use cairo::{Context, Format, ImageSurface};
use uuid::Uuid;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use log::{debug, info, warn, error};
use crate::core::document::Document;

/// Represents a layer in the image
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub visible: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: u32,
    pub height: u32,
}

/// Layer blend modes for compositing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
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
}

impl Layer {
    /// Create a new empty layer with the given dimensions
    pub fn new(width: u32, height: u32, name: String) -> Self {
        info!("Creating new layer: {} ({}x{})", name, width, height);
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            image: ImageBuffer::new(width, height),
            visible: true,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            x_offset: 0,
            y_offset: 0,
            width,
            height,
        }
    }
    
    /// Create a layer from an existing image
    pub fn from_image(image: ImageBuffer<Rgba<u8>, Vec<u8>>, name: String) -> Self {
        info!("Creating layer from image: {}", name);
        let width = image.width();
        let height = image.height();
        debug!("Layer dimensions: {}x{}", width, height);
        
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            image,
            visible: true,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            x_offset: 0,
            y_offset: 0,
            width,
            height,
        }
    }
    
    /// Create a layer from a document
    pub fn from_document(doc: &Document) -> Self {
        info!("Creating layer from document");
        if let Some(image) = doc.get_image() {
            debug!("Got image from document");
            Self::from_image(image, "Background".to_string())
        } else {
            warn!("No image in document, creating empty layer");
            Self::new(doc.width, doc.height, "Background".to_string())
        }
    }
    
    /// Create a duplicate of this layer
    pub fn duplicate(&self, new_name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: new_name,
            image: self.image.clone(),
            visible: self.visible,
            opacity: self.opacity,
            blend_mode: self.blend_mode,
            x_offset: self.x_offset,
            y_offset: self.y_offset,
            width: self.width,
            height: self.height,
        }
    }
    
    /// Resize the layer to the given dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        let mut new_image = ImageBuffer::new(width, height);
        
        // Copy the existing image data, if it fits
        let copy_width = width.min(self.image.width());
        let copy_height = height.min(self.image.height());
        
        for y in 0..copy_height {
            for x in 0..copy_width {
                let pixel = self.image.get_pixel(x, y);
                new_image.put_pixel(x, y, *pixel);
            }
        }
        
        self.image = new_image;
    }
    
    /// Crop the layer to the given rectangle
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
        let mut new_image = ImageBuffer::new(width, height);
        
        // Copy the selected portion of the image
        for new_y in 0..height {
            for new_x in 0..width {
                let src_x = x + new_x;
                let src_y = y + new_y;
                
                if src_x < self.image.width() && src_y < self.image.height() {
                    let pixel = self.image.get_pixel(src_x, src_y);
                    new_image.put_pixel(new_x, new_y, *pixel);
                }
            }
        }
        
        self.image = new_image;
        
        // Update offset
        self.x_offset -= x as i32;
        self.y_offset -= y as i32;
    }
    
    /// Clear the layer (set all pixels to transparent)
    pub fn clear(&mut self) {
        for pixel in self.image.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 0]);
        }
    }
    
    /// Set a pixel in the layer
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        if x < self.image.width() && y < self.image.height() {
            self.image.put_pixel(x, y, color);
        }
    }
    
    /// Get a pixel from the layer
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Rgba<u8>> {
        if x < self.image.width() && y < self.image.height() {
            Some(*self.image.get_pixel(x, y))
        } else {
            None
        }
    }
    
    /// Set the opacity of the layer
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.max(0.0).min(1.0);
    }
    
    /// Set the visibility of the layer
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    /// Set the blend mode of the layer
    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.blend_mode = blend_mode;
    }
    
    /// Set the offset of the layer
    pub fn set_offset(&mut self, x_offset: i32, y_offset: i32) {
        self.x_offset = x_offset;
        self.y_offset = y_offset;
    }
    
    /// Render the layer to a Cairo context
    pub fn render(&self, cr: &Context, width: u32, height: u32) {
        if !self.visible {
            debug!("Layer {} is not visible, skipping render", self.name);
            return;
        }
        
        debug!("Rendering layer: {}", self.name);
        
        // Create a surface from the image data
        let data = self.image.as_raw().to_vec();
        let stride = cairo::Format::Rgb24.stride_for_width(self.width as u32)
            .expect("Failed to calculate stride");
            
        unsafe {
            if let Ok(surface) = ImageSurface::create_for_data(
                data,
                Format::Rgb24,
                self.width as i32,
                self.height as i32,
                stride
            ) {
                // Save the current state
                cr.save().expect("Failed to save context state");
                
                // Set the opacity
                if self.opacity < 1.0 {
                    cr.push_group();
                }
                
                // Draw the image
                cr.set_source_surface(&surface, 0.0, 0.0)
                    .expect("Failed to set source surface");
                cr.paint().expect("Failed to paint surface");
                
                // Apply opacity if needed
                if self.opacity < 1.0 {
                    cr.pop_group_to_source().expect("Failed to pop group");
                    cr.paint_with_alpha(self.opacity as f64).expect("Failed to paint with alpha");
                }
                
                // Restore the state
                cr.restore().expect("Failed to restore context state");
                
                debug!("Layer {} rendered successfully", self.name);
            } else {
                error!("Failed to create surface for layer {}", self.name);
            }
        }
    }
}

/// Manages multiple layers in an image
#[derive(Debug, PartialEq, Clone)]
pub struct LayerManager {
    layers: Vec<Layer>,
    active_layer_index: usize,
}

impl LayerManager {
    /// Create a new layer manager
    pub fn new() -> Self {
        info!("Creating new layer manager");
        Self {
            layers: Vec::new(),
            active_layer_index: 0,
        }
    }
    
    /// Add a new layer
    pub fn add_layer(&mut self, layer: Layer) -> usize {
        info!("Adding layer: {}", layer.name);
        self.layers.push(layer);
        let index = self.layers.len() - 1;
        self.active_layer_index = index;
        index
    }
    
    /// Get a reference to a layer
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }
    
    /// Get a mutable reference to a layer
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
    
    /// Replace a layer at the given index
    pub fn set_layer(&mut self, index: usize, layer: Layer) {
        if index < self.layers.len() {
            self.layers[index] = layer;
        }
    }
    
    /// Remove a layer at the given index
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index < self.layers.len() {
            let layer = self.layers.remove(index);
            
            // Update active layer index
            if self.active_layer_index >= self.layers.len() && self.layers.len() > 0 {
                self.active_layer_index = self.layers.len() - 1;
            }
            
            Some(layer)
        } else {
            None
        }
    }
    
    /// Move a layer from one position to another
    pub fn move_layer(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.layers.len() || to_index >= self.layers.len() {
            return false;
        }
        
        if from_index == to_index {
            return true;
        }
        
        let layer = self.layers.remove(from_index);
        self.layers.insert(to_index, layer);
        
        // Update active layer index if needed
        if self.active_layer_index == from_index {
            self.active_layer_index = to_index;
        } else if from_index < self.active_layer_index && to_index >= self.active_layer_index {
            self.active_layer_index -= 1;
        } else if from_index > self.active_layer_index && to_index <= self.active_layer_index {
            self.active_layer_index += 1;
        }
        
        true
    }
    
    /// Set the active layer
    pub fn set_active_layer(&mut self, index: usize) -> bool {
        if index < self.layers.len() {
            self.active_layer_index = index;
            true
        } else {
            false
        }
    }
    
    /// Get the active layer index
    pub fn get_active_layer_index(&self) -> usize {
        self.active_layer_index
    }
    
    /// Get a reference to the active layer
    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.layers.get(self.active_layer_index)
    }
    
    /// Get a mutable reference to the active layer
    pub fn get_active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers.get_mut(self.active_layer_index)
    }
    
    /// Get the number of layers
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
    
    /// Resize all layers to the given dimensions
    pub fn resize_all_layers(&mut self, width: u32, height: u32) {
        for layer in &mut self.layers {
            layer.resize(width, height);
        }
    }
    
    /// Crop all layers to the given rectangle
    pub fn crop_all_layers(&mut self, x: u32, y: u32, width: u32, height: u32) {
        for layer in &mut self.layers {
            layer.crop(x, y, width, height);
        }
    }
    
    /// Merge the visible layers into a single image
    pub fn flatten(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        debug!("Flattening layers");
        if self.layers.is_empty() {
            error!("No layers to flatten");
            return ImageBuffer::new(1, 1);
        }
        
        // Use the first layer's dimensions
        let width = self.layers[0].width;
        let height = self.layers[0].height;
        debug!("Creating flattened image with dimensions {}x{}", width, height);
        
        let mut result = ImageBuffer::new(width, height);
        
        // Composite all visible layers
        for layer in &self.layers {
            if layer.visible {
                debug!("Compositing layer: {}", layer.name);
                for (x, y, pixel) in result.enumerate_pixels_mut() {
                    if x < layer.width && y < layer.height {
                        let src_pixel = layer.image.get_pixel(x, y);
                        // Simple alpha compositing
                        let alpha = (src_pixel[3] as f64 * layer.opacity) as u8;
                        *pixel = Rgba([
                            src_pixel[0],
                            src_pixel[1],
                            src_pixel[2],
                            alpha,
                        ]);
                    }
                }
            }
        }
        
        debug!("Layers flattened successfully");
        result
    }
    
    /// Render all layers to a Cairo context
    pub fn render(&self, context: &Context, _width: u32, _height: u32) {
        // Draw the layers bottom to top
        for layer in &self.layers {
            layer.render(context, _width, _height);
        }
    }

    pub fn get_layers(&self) -> &[Layer] {
        &self.layers
    }
}

/// Blend two pixels according to the specified blend mode and opacity
fn blend_pixels(dst: &Rgba<u8>, src: &Rgba<u8>, blend_mode: BlendMode, opacity: f32) -> Rgba<u8> {
    // If source is fully transparent, return destination unchanged
    if src[3] == 0 {
        return *dst;
    }
    
    // If destination is fully transparent and blend mode is Normal, return source
    if dst[3] == 0 && blend_mode == BlendMode::Normal {
        return Rgba([
            src[0],
            src[1],
            src[2],
            (src[3] as f32 * opacity) as u8,
        ]);
    }
    
    // Convert to float for calculations
    let src_r = src[0] as f32 / 255.0;
    let src_g = src[1] as f32 / 255.0;
    let src_b = src[2] as f32 / 255.0;
    let src_a = src[3] as f32 / 255.0 * opacity;
    
    let dst_r = dst[0] as f32 / 255.0;
    let dst_g = dst[1] as f32 / 255.0;
    let dst_b = dst[2] as f32 / 255.0;
    let dst_a = dst[3] as f32 / 255.0;
    
    // Calculate result color based on blend mode
    let (_result_r, _result_g, _result_b) = match blend_mode {
        BlendMode::Normal => (src_r, src_g, src_b),
        BlendMode::Multiply => (src_r * dst_r, src_g * dst_g, src_b * dst_b),
        BlendMode::Screen => (
            1.0 - (1.0 - src_r) * (1.0 - dst_r),
            1.0 - (1.0 - src_g) * (1.0 - dst_g),
            1.0 - (1.0 - src_b) * (1.0 - dst_b)
        ),
        BlendMode::Overlay => (
            if dst_r < 0.5 { 2.0 * src_r * dst_r } else { 1.0 - 2.0 * (1.0 - src_r) * (1.0 - dst_r) },
            if dst_g < 0.5 { 2.0 * src_g * dst_g } else { 1.0 - 2.0 * (1.0 - src_g) * (1.0 - dst_g) },
            if dst_b < 0.5 { 2.0 * src_b * dst_b } else { 1.0 - 2.0 * (1.0 - src_b) * (1.0 - dst_b) }
        ),
        BlendMode::Darken => (
            src_r.min(dst_r),
            src_g.min(dst_g),
            src_b.min(dst_b)
        ),
        BlendMode::Lighten => (
            src_r.max(dst_r),
            src_g.max(dst_g),
            src_b.max(dst_b)
        ),
        // For more complex blend modes, we simplify to Normal for now
        _ => (src_r, src_g, src_b),
    };
    
    // Apply alpha compositing
    let out_a = src_a + dst_a * (1.0 - src_a);
    
    // If the result is fully transparent, return transparent
    if out_a <= 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    
    // Apply alpha-weighted blend of colors
    let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a;
    let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a;
    let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a;
    
    // Convert back to u8
    Rgba([
        (out_r * 255.0).min(255.0).max(0.0) as u8,
        (out_g * 255.0).min(255.0).max(0.0) as u8,
        (out_b * 255.0).min(255.0).max(0.0) as u8,
        (out_a * 255.0).min(255.0).max(0.0) as u8,
    ])
} 