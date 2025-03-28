use gtk4::prelude::*;
use gtk4::{DrawingArea, EventControllerMotion, GestureClick, GestureDrag, Orientation};
use cairo::{Context, Format, ImageSurface};
use std::cell::RefCell;
use std::rc::Rc;
use crate::vector::{Point, VectorPath, VectorDocument, VectorShape};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::collections::HashMap;
use crate::core::layer::{Layer, LayerManager, Selection};

/// Available tools for image editing
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tool {
    Selection,
    Transform,
    Vector,
    Paint,
    Eraser,
    Clone,
    Healing,
    Text,
    Zoom,
    Hand,
}

/// Selection type for the Selection tool
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectionType {
    Rectangle,
    Ellipse,
    Freehand,
    MagicWand,
    Color,
}

/// Settings for brush-based tools
#[derive(Debug, Clone)]
pub struct BrushSettings {
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
    pub flow: f64,
    pub pressure_sensitivity: bool,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 20.0,
            hardness: 0.8,
            opacity: 1.0,
            flow: 1.0,
            pressure_sensitivity: true,
        }
    }
}

/// Represents the state of the canvas, including view
#[derive(Clone)]
pub struct Canvas {
    /// Width of the canvas
    pub width: u32,
    /// Height of the canvas
    pub height: u32,
    /// Layer manager for the canvas
    pub layer_manager: LayerManager,
    /// Current selection
    pub selection: Option<Selection>,
    /// Zoom level (1.0 = 100%)
    pub zoom: f64,
    /// Horizontal offset for panning
    pub offset_x: f64,
    /// Vertical offset for panning
    pub offset_y: f64,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub vector_document: Option<VectorDocument>,
    pub has_vector_mode: bool,
}

/// Represents a selection in the image
#[derive(Clone)]
pub struct Selection {
    /// X coordinate of the selection
    pub x: i32,
    /// Y coordinate of the selection
    pub y: i32,
    /// Width of the selection
    pub width: i32,
    /// Height of the selection
    pub height: i32,
    /// Mask bitmap for the selection
    pub mask: Option<image::GrayImage>,
}

impl Selection {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            mask: None,
        }
    }
    
    pub fn with_mask(x: i32, y: i32, width: i32, height: i32, mask: image::GrayImage) -> Self {
        Self {
            x,
            y,
            width,
            height,
            mask: Some(mask),
        }
    }
    
    /// Get the bounds of the selection as a rectangle
    pub fn get_bounds(&self) -> Option<cairo::Rectangle> {
        if self.width <= 0 || self.height <= 0 {
            return None;
        }
        
        // Create a cairo::Rectangle using the tuple struct syntax
        Some(cairo::Rectangle::new(
            self.x as f64,
            self.y as f64,
            self.width as f64,
            self.height as f64
        ))
    }
    
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        if x < self.x || y < self.y || x >= self.x + self.width || y >= self.y + self.height {
            return false;
        }
        
        if let Some(mask) = &self.mask {
            // Check the mask for partial selections
            let local_x = (x - self.x) as u32;
            let local_y = (y - self.y) as u32;
            
            if local_x < mask.width() && local_y < mask.height() {
                let pixel = mask.get_pixel(local_x, local_y);
                return pixel[0] > 0;
            }
        }
        
        true
    }
    
    /// Check if the selection contains a point with u32 coordinates
    pub fn contains(&self, x: u32, y: u32) -> bool {
        // Convert u32 to i32 for our internal representation
        self.contains_point(x as i32, y as i32)
    }
}

impl Canvas {
    /// Create a new canvas with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let mut layer_manager = LayerManager::new();
        
        // Create a default background layer
        let background = Layer::new(width, height, "Background".to_string());
        layer_manager.add_layer(background);
        
        Self {
            width,
            height,
            layer_manager,
            selection: None,
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            vector_document: None,
            has_vector_mode: false,
        }
    }
    
    /// Create a canvas from an existing image
    pub fn from_image(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        let width = image.width();
        let height = image.height();
        
        let mut layer_manager = LayerManager::new();
        
        // Create a layer from the image
        let layer = Layer::from_image(image, "Background".to_string());
        layer_manager.add_layer(layer);
        
        Self {
            width,
            height,
            layer_manager,
            selection: None,
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            vector_document: None,
            has_vector_mode: false,
        }
    }
    
    /// Resize the canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.layer_manager.resize_all_layers(width, height);
        
        // Update vector document
        if let Some(vector_doc) = &mut self.vector_document {
            // For now, we just create a new document with the new size
            // In a real implementation, you'd want to keep the existing vector elements
            self.vector_document = Some(VectorDocument::new(
                width as i32,
                height as i32
            ));
        }
    }
    
    /// Crop the canvas to the selection or given rectangle
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.layer_manager.crop_all_layers(x, y, width, height);
        self.selection = None; // Clear selection after crop
        
        // Update vector document
        if let Some(vector_doc) = &mut self.vector_document {
            // For now, we just create a new document with the new size
            // In a real implementation, you'd want to adjust the existing vector elements
            self.vector_document = Some(VectorDocument::new(
                width as i32,
                height as i32
            ));
        }
    }
    
    /// Crop to the current selection
    pub fn crop_to_selection(&mut self) -> bool {
        if let Some(selection) = &self.selection {
            let x = selection.x.max(0) as u32;
            let y = selection.y.max(0) as u32;
            let width = selection.width as u32;
            let height = selection.height as u32;
            
            self.crop(x, y, width, height);
            return true;
        }
        false
    }
    
    /// Set the selection for the canvas
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
    }
    
    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
    
    /// Check if a point is within the selection
    pub fn is_selected(&self, x: u32, y: u32) -> bool {
        if let Some(selection) = &self.selection {
            selection.contains(x, y)
        } else {
            true // If no selection, entire canvas is considered selected
        }
    }
    
    /// Set the zoom level
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.max(0.1).min(32.0); // Limit zoom range
        
        // No longer needed - we'll use the canvas's zoom value directly when rendering
        // if let Some(vector_doc) = &mut self.vector_document {
        //     vector_doc.set_zoom(self.zoom);
        // }
    }
    
    /// Zoom in (multiply zoom by factor)
    pub fn zoom_in(&mut self, factor: f64) {
        self.zoom *= factor;
        self.zoom = self.zoom.max(0.1).min(32.0); // Limit zoom range
        
        // No longer needed - we'll use the canvas's zoom value directly when rendering
        // if let Some(vector_doc) = &mut self.vector_document {
        //     vector_doc.set_zoom(self.zoom);
        // }
    }
    
    /// Zoom out (divide zoom by factor)
    pub fn zoom_out(&mut self, factor: f64) {
        self.zoom /= factor;
        self.zoom = self.zoom.max(0.1).min(32.0); // Limit zoom range
        
        // No longer needed - we'll use the canvas's zoom value directly when rendering
        // if let Some(vector_doc) = &mut self.vector_document {
        //     vector_doc.set_zoom(self.zoom);
        // }
    }
    
    /// Center the view on the canvas
    pub fn center_view(&mut self, view_width: u32, view_height: u32) {
        // Calculate offsets to center the image in the view
        let scaled_width = self.width as f64 * self.zoom;
        let scaled_height = self.height as f64 * self.zoom;
        
        self.offset_x = (view_width as f64 - scaled_width) / 2.0;
        self.offset_y = (view_height as f64 - scaled_height) / 2.0;
        
        // No longer needed - we'll use the canvas's offset values directly when rendering
        // if let Some(vector_doc) = &mut self.vector_document {
        //     vector_doc.set_view_offset(Point::new(self.offset_x, self.offset_y));
        // }
    }
    
    /// Pan the view
    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        self.offset_x += delta_x;
        self.offset_y += delta_y;
        
        // No longer needed - we'll use the canvas's offset values directly when rendering
        // if let Some(vector_doc) = &mut self.vector_document {
        //     vector_doc.set_view_offset(Point::new(self.offset_x, self.offset_y));
        // }
    }
    
    /// Render the canvas to a Cairo context
    pub fn render(&self, context: &Context, view_width: u32, view_height: u32) {
        // Clear the background (using a checkerboard pattern for transparency)
        self.render_transparency_pattern(context, view_width, view_height);
        
        // Set up the transformation for zoom and pan
        context.save();
        context.translate(self.offset_x, self.offset_y);
        context.scale(self.zoom, self.zoom);
        
        // Render all layers
        self.layer_manager.render(context, self.width, self.height);
        
        // Render the selection if present
        if let Some(selection) = &self.selection {
            // Draw a rectangle around the selection bounds instead of using render_outline
            if let Some(bounds) = selection.get_bounds() {
                context.set_source_rgba(0.0, 0.5, 1.0, 0.8);
                context.set_line_width(1.0);
                context.rectangle(bounds.x(), bounds.y(), bounds.width(), bounds.height());
                context.stroke();
                
                // Draw handles at the corners and midpoints
                context.set_source_rgba(1.0, 1.0, 1.0, 0.9);
                let handle_size = 5.0;
                let midpoints = [
                    (bounds.x(), bounds.y()), // Top-left
                    (bounds.x() + bounds.width() / 2.0, bounds.y()), // Top-center
                    (bounds.x() + bounds.width(), bounds.y()), // Top-right
                    (bounds.x() + bounds.width(), bounds.y() + bounds.height() / 2.0), // Right-center
                    (bounds.x() + bounds.width(), bounds.y() + bounds.height()), // Bottom-right
                    (bounds.x() + bounds.width() / 2.0, bounds.y() + bounds.height()), // Bottom-center
                    (bounds.x(), bounds.y() + bounds.height()), // Bottom-left
                    (bounds.x(), bounds.y() + bounds.height() / 2.0), // Left-center
                ];
                
                for (x, y) in midpoints.iter() {
                    context.rectangle(
                        x - handle_size / 2.0,
                        y - handle_size / 2.0,
                        handle_size,
                        handle_size
                    );
                    context.fill();
                }
            }
        }
        
        context.restore();
    }
    
    /// Render a checkerboard pattern for transparency
    fn render_transparency_pattern(&self, context: &Context, width: u32, height: u32) {
        let cell_size = 16.0; // Size of each checkerboard cell
        
        context.save();
        
        // Fill the entire view with white
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint();
        
        // Draw the checkerboard pattern
        context.set_source_rgb(0.8, 0.8, 0.8); // Light gray
        
        for y in (0..height).step_by(cell_size as usize * 2) {
            for x in (0..width).step_by(cell_size as usize * 2) {
                // Draw the gray squares in a checkerboard pattern
                context.rectangle(x as f64, y as f64, cell_size, cell_size);
                context.rectangle(
                    x as f64 + cell_size,
                    y as f64 + cell_size,
                    cell_size,
                    cell_size,
                );
            }
        }
        
        context.fill();
        context.restore();
    }
    
    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_x: f64, screen_y: f64) -> Point {
        let canvas_x = (screen_x - self.offset_x) / self.zoom;
        let canvas_y = (screen_y - self.offset_y) / self.zoom;
        Point::new(canvas_x, canvas_y)
    }
    
    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_x: f64, canvas_y: f64) -> Point {
        let screen_x = canvas_x * self.zoom + self.offset_x;
        let screen_y = canvas_y * self.zoom + self.offset_y;
        Point::new(screen_x, screen_y)
    }
    
    /// Check if a point is within the canvas bounds
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= 0.0 && x < self.width as f64 && y >= 0.0 && y < self.height as f64
    }
    
    /// Get the active layer
    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.layer_manager.get_active_layer()
    }
    
    /// Get a mutable reference to the active layer
    pub fn get_active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layer_manager.get_active_layer_mut()
    }
    
    /// Export the canvas as a flattened image
    pub fn export(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        self.layer_manager.flatten()
    }
    
    /// Export the canvas as a DynamicImage
    pub fn export_dynamic(&self) -> DynamicImage {
        DynamicImage::ImageRgba8(self.export())
    }
    
    /// Toggle vector mode
    pub fn toggle_vector_mode(&mut self) {
        self.has_vector_mode = !self.has_vector_mode;
    }
    
    /// Render the canvas to a Cairo context
    pub fn render_to_cairo_context(&self, context: &Context) {
        // Save the context state
        context.save();
        
        // Apply zoom and offset
        context.translate(self.offset_x, self.offset_y);
        context.scale(self.zoom, self.zoom);
        
        // Clear the background
        context.set_source_rgb(0.5, 0.5, 0.5);
        context.paint();
        
        // Draw checkerboard pattern for transparency
        self.draw_transparency_checkerboard(context);
        
        // Render regular layers
        if !self.has_vector_mode {
            self.layer_manager.render(context, self.width, self.height);
            
            // Render selection outline
            if let Some(selection) = &self.selection {
                // TODO: selection.render_outline(context);
                // For now, just draw a rectangle around the selection bounds
                if let Some(bounds) = selection.get_bounds() {
                    context.set_source_rgba(0.0, 0.5, 1.0, 0.8);
                    context.set_line_width(1.0);
                    context.rectangle(bounds.x(), bounds.y(), bounds.width(), bounds.height());
                    context.stroke();
                }
            }
        } else {
            // Render vector document in vector mode
            if let Some(vector_doc) = &self.vector_document {
                // Pass current transformation to vector_doc.draw
                vector_doc.draw(context);
            }
        }
        
        // Restore the context state
        context.restore();
    }
    
    /// Draw a checkerboard pattern to represent transparency
    fn draw_transparency_checkerboard(&self, context: &Context) {
        let cell_size = 8.0;
        let color1 = (0.8, 0.8, 0.8);
        let color2 = (0.7, 0.7, 0.7);
        
        context.save();
        
        // Calculate number of cells needed to cover the canvas
        let cell_cols = (self.width as f64 / cell_size).ceil() as i32;
        let cell_rows = (self.height as f64 / cell_size).ceil() as i32;
        
        for row in 0..cell_rows {
            for col in 0..cell_cols {
                let is_color1 = (row + col) % 2 == 0;
                let (r, g, b) = if is_color1 { color1 } else { color2 };
                
                context.set_source_rgb(r, g, b);
                context.rectangle(
                    col as f64 * cell_size,
                    row as f64 * cell_size,
                    cell_size,
                    cell_size
                );
                context.fill();
            }
        }
        
        context.restore();
    }
    
    /// Export the canvas to a file
    pub fn export_to_file(&self, path: &str) -> Result<(), String> {
        let image = self.export_dynamic();
        image.save(path).map_err(|e| e.to_string())
    }
    
    /// Export the vector document to SVG
    pub fn export_svg(&self, path: &str) -> Result<(), String> {
        if let Some(_vector_doc) = &self.vector_document {
            // Since vector_doc.export_as_svg doesn't exist, implement a basic stub
            // In a real implementation, we'd either:
            // 1. Add the export_as_svg method to VectorDocument
            // 2. Create the SVG here directly using the vector document data
            
            // For now, just create a basic SVG file for demonstration
            use std::fs::File;
            use std::io::Write;
            
            let mut file = File::create(path).map_err(|e| e.to_string())?;
            
            // Write a basic SVG header
            let svg_header = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
                <svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
                <rect width="100%" height="100%" fill="white"/>
                <!-- This is a stub SVG export -->
                <text x="50%" y="50%" font-family="Arial" font-size="24" text-anchor="middle">
                SVG Export Stub
                </text>
                </svg>"#,
                self.width, self.height
            );
            
            file.write_all(svg_header.as_bytes()).map_err(|e| e.to_string())?;
            
            Ok(())
        } else {
            Err("No vector document available".to_string())
        }
    }
    
    /// Gets the current mouse position
    pub fn get_mouse_position(&self) -> (f64, f64) {
        (self.mouse_x, self.mouse_y)
    }
} 