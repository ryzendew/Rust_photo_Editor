use gtk4::prelude::*;
use gtk4::{DrawingArea, GestureZoom, EventControllerMotion, EventControllerScroll, GestureDrag};
use cairo::{Context, Format, ImageSurface};
use std::cell::RefCell;
use std::rc::Rc;
use log::{debug, info, warn, error};

use crate::core::document::Document;
use crate::core::layer::{Layer, LayerManager};
use crate::tools::{ToolType, ToolManager, ToolContext};
use crate::vector::{VectorDocument, Point};
use crate::ui::ColorPicker;
use crate::core::selection::Selection;

#[derive(Clone)]
pub struct Canvas {
    pub widget: DrawingArea,
    pub document: Option<Document>,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub tool_manager: Option<ToolManager>,
    pub vector_document: Option<VectorDocument>,
    pub layer_manager: Option<LayerManager>,
    pub selection: Option<Selection>,
    pub tool_context: Option<ToolContext>,
    pub color_picker: ColorPicker,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub is_dragging: bool,
    pub drag_start_x: f64,
    pub drag_start_y: f64,
}

impl Canvas {
    pub fn new() -> Self {
        let widget = DrawingArea::new();
        widget.set_hexpand(true);
        widget.set_vexpand(true);
        widget.set_can_focus(true);
        
        Canvas {
            widget,
            document: None,
            width: 800,
            height: 600,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            tool_manager: None,
            vector_document: None,
            layer_manager: None,
            selection: None,
            tool_context: None,
            color_picker: ColorPicker::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
        }
    }
    
    pub fn set_document(&mut self, document: Document) {
        self.document = Some(document);
        self.widget.queue_draw();
    }
    
    pub fn set_vector_document(&mut self, document: VectorDocument) {
        self.vector_document = Some(document);
        self.widget.queue_draw();
    }
    
    pub fn set_layer_manager(&mut self, layer_manager: LayerManager) {
        self.layer_manager = Some(layer_manager);
        self.widget.queue_draw();
    }
    
    pub fn set_tool_context(&mut self, tool_context: ToolContext) {
        self.tool_context = Some(tool_context);
    }
    
    pub fn draw(&self, cr: &Context) {
        // Clear the canvas
        cr.set_source_rgb(0.9, 0.9, 0.9);
        cr.paint().expect("Failed to paint background");
        
        // Draw a checkerboard pattern for transparent areas
        self.draw_checkerboard(cr);
        
        // Apply transformations for zoom and pan
        cr.translate(self.offset_x, self.offset_y);
        cr.scale(self.scale, self.scale);
        
        // Draw the document if available
        if let Some(doc) = &self.document {
            self.draw_document(cr, doc);
        } else if let Some(vector_doc) = &self.vector_document {
            self.draw_vector_document(cr, vector_doc);
        } else {
            // Draw a placeholder if no document
            self.draw_placeholder(cr);
        }
    }
    
    fn draw_vector_document(&self, cr: &Context, doc: &VectorDocument) {
        // Draw each layer in the vector document
        for layer in doc.get_layers() {
            layer.draw(cr);
        }
    }
    
    fn draw_checkerboard(&self, cr: &Context) {
        let cell_size = 10.0;
        let width = self.width as f64;
        let height = self.height as f64;
        
        for y in (0..(height as i32)).step_by(cell_size as usize) {
            for x in (0..(width as i32)).step_by(cell_size as usize) {
                if (x / cell_size as i32 + y / cell_size as i32) % 2 == 0 {
                    cr.set_source_rgb(0.8, 0.8, 0.8);
                } else {
                    cr.set_source_rgb(0.7, 0.7, 0.7);
                }
                cr.rectangle(x as f64, y as f64, cell_size, cell_size);
                cr.fill().expect("Failed to fill checkerboard cell");
            }
        }
    }
    
    fn draw_placeholder(&self, cr: &Context) {
        cr.set_source_rgb(0.8, 0.8, 0.8);
        cr.rectangle(0.0, 0.0, self.width as f64, self.height as f64);
        cr.fill().expect("Failed to fill placeholder");
        
        cr.set_source_rgb(0.4, 0.4, 0.4);
        cr.set_font_size(24.0);
        let text = "No document loaded";
        let extents = cr.text_extents(text).expect("Failed to get text extents");
        let x = (self.width as f64 - extents.width()) / 2.0;
        let y = (self.height as f64 + extents.height()) / 2.0;
        cr.move_to(x, y);
        cr.show_text(text).expect("Failed to show text");
    }
    
    fn draw_document(&self, cr: &Context, doc: &Document) {
        // Draw each layer if layer manager is available
        if let Some(layer_manager) = &self.layer_manager {
            for layer in layer_manager.get_layers() {
                if layer.visible {
                    layer.render(cr, self.width as u32, self.height as u32);
                }
            }
        }
    }
    
    pub fn clone(&self) -> Self {
        Canvas {
            widget: self.widget.clone(),
            document: self.document.clone(),
            width: self.width,
            height: self.height,
            scale: self.scale,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            tool_manager: self.tool_manager.clone(),
            vector_document: self.vector_document.clone(),
            layer_manager: self.layer_manager.clone(),
            selection: self.selection.clone(),
            tool_context: self.tool_context.clone(),
            color_picker: self.color_picker.clone(),
            mouse_x: self.mouse_x,
            mouse_y: self.mouse_y,
            is_dragging: self.is_dragging,
            drag_start_x: self.drag_start_x,
            drag_start_y: self.drag_start_y,
        }
    }
    
    pub fn get_widget(&self) -> DrawingArea {
        self.widget.clone()
    }
    
    pub fn set_tool_manager(&mut self, tool_manager: Option<ToolManager>) {
        info!("Setting tool manager in canvas");
        self.tool_manager = tool_manager;
    }
    
    pub fn zoom_to_fit(&mut self) {
        info!("Zooming to fit");
        
        // Get the widget dimensions
        let width = self.widget.width();
        let height = self.widget.height();
        
        // Calculate scale to fit
        let scale_x = width as f64 / self.width as f64;
        let scale_y = height as f64 / self.height as f64;
        self.scale = scale_x.min(scale_y) * 0.9; // 90% to leave a margin
        
        // Center the canvas
        self.center();
        
        // Redraw
        self.widget.queue_draw();
    }
    
    pub fn zoom_to_actual(&mut self) {
        info!("Zooming to actual size (100%)");
        
        // Reset scale
        self.scale = 1.0;
        
        // Center the canvas
        self.center();
        
        // Redraw
        self.widget.queue_draw();
    }
    
    pub fn center(&mut self) {
        // Get the widget dimensions
        let width = self.widget.width();
        let height = self.widget.height();
        
        // Calculate center position
        self.offset_x = (width as f64 - self.width as f64 * self.scale) / 2.0;
        self.offset_y = (height as f64 - self.height as f64 * self.scale) / 2.0;
    }
} 