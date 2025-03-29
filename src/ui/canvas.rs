use gtk4::prelude::*;
use gtk4::{DrawingArea, EventControllerMotion, EventControllerScroll, GestureZoom, GestureDrag};
use cairo::{Context, Format, ImageSurface};
use std::cell::RefCell;
use std::rc::Rc;
use log::{debug, info, error};

use crate::core::document::Document;
use crate::core::layer::{Layer, LayerManager};
use crate::tools::{ToolType, ToolManager, ToolContext};
use crate::vector::{VectorDocument, Point};
use crate::ui::ColorPicker;
use crate::core::selection::Selection;

pub struct Canvas {
    drawing_area: DrawingArea,
    document: Option<Document>,
    layer_manager: Option<LayerManager>,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    pub width: i32,
    pub height: i32,
    tool_manager: Option<ToolManager>,
    vector_document: Option<VectorDocument>,
    selection: Option<Selection>,
    tool_context: Option<ToolContext>,
    color_picker: ColorPicker,
    mouse_x: f64,
    mouse_y: f64,
    is_dragging: bool,
    drag_start_x: f64,
    drag_start_y: f64,
}

impl Canvas {
    pub fn new() -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_content_width(800);
        drawing_area.set_content_height(600);

        let canvas = Self {
            drawing_area,
            document: None,
            layer_manager: None,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            width: 800,
            height: 600,
            tool_manager: None,
            vector_document: None,
            selection: None,
            tool_context: None,
            color_picker: ColorPicker::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_dragging: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
        };

        canvas.setup_drawing();
        canvas.setup_input_handlers();
        canvas
    }

    fn setup_drawing(&self) {
        let canvas_ref = self.clone();
        self.drawing_area.set_draw_func(move |_, cr, width, height| {
            canvas_ref.draw(cr, width, height);
        });
    }

    fn setup_input_handlers(&self) {
        // Zoom gesture
        let zoom = GestureZoom::new();
        self.drawing_area.add_controller(zoom.clone());
        let canvas_ref = Rc::new(RefCell::new(self.clone()));
        zoom.connect_scale_changed(move |_, scale| {
            canvas_ref.borrow_mut().on_zoom(scale);
        });

        // Pan gesture
        let drag = GestureDrag::new();
        self.drawing_area.add_controller(drag.clone());
        let canvas_ref = Rc::new(RefCell::new(self.clone()));
        drag.connect_drag_begin(move |_, x, y| {
            canvas_ref.borrow_mut().on_drag_begin(x, y);
        });

        let canvas_ref = Rc::new(RefCell::new(self.clone()));
        drag.connect_drag_update(move |_, x, y| {
            canvas_ref.borrow_mut().on_drag_update(x, y);
        });
    }

    pub fn set_document(&mut self, document: Option<Document>) {
        self.document = document;
        if let Some(doc) = &self.document {
            self.width = doc.width as i32;
            self.height = doc.height as i32;
            self.drawing_area.set_content_width(self.width);
            self.drawing_area.set_content_height(self.height);
            
            let mut layer_manager = LayerManager::new();
            layer_manager.add_layer(Layer::from_document(doc));
            self.layer_manager = Some(layer_manager);
        }
        self.drawing_area.queue_draw();
    }

    pub fn set_vector_document(&mut self, document: VectorDocument) {
        self.vector_document = Some(document);
        self.drawing_area.queue_draw();
    }

    pub fn set_tool_context(&mut self, tool_context: ToolContext) {
        self.tool_context = Some(tool_context);
    }

    pub fn draw(&self, cr: &Context, width: i32, height: i32) {
        // Clear background
        cr.set_source_rgb(0.2, 0.2, 0.2);
        cr.paint().expect("Failed to paint background");

        if let Some(doc) = &self.document {
            debug!("Drawing document");
            self.draw_document(cr, doc);
        } else if let Some(vector_doc) = &self.vector_document {
            debug!("Drawing vector document");
            self.draw_vector_document(cr, vector_doc);
        } else {
            debug!("Drawing placeholder - no document loaded");
            self.draw_placeholder(cr, width, height);
        }
    }

    fn draw_vector_document(&self, cr: &Context, doc: &VectorDocument) {
        // Draw each layer in the vector document
        for layer in doc.get_layers() {
            layer.draw(cr);
        }
    }

    fn draw_document(&self, cr: &Context, doc: &Document) {
        debug!("Drawing document with dimensions: {}x{}", doc.width, doc.height);

        // Apply transformations
        cr.save().expect("Failed to save context state");
        cr.translate(self.offset_x, self.offset_y);
        cr.scale(self.scale, self.scale);

        if let Some(layer_manager) = &self.layer_manager {
            debug!("Drawing layers from layer manager");
            for layer in layer_manager.get_layers() {
                layer.render(cr, self.width as u32, self.height as u32);
            }
        } else if let Some(image) = doc.get_image() {
            debug!("Rendering document image directly");
            let data = image.as_raw().to_vec();
            let surface = ImageSurface::create_for_data(
                data,
                Format::Rgb24,
                doc.width as i32,
                doc.height as i32,
                doc.width as i32 * 4,
            ).expect("Failed to create surface");

            cr.set_source_surface(&surface, 0.0, 0.0)
                .expect("Failed to set source surface");
            cr.paint().expect("Failed to paint image");
        }

        cr.restore().expect("Failed to restore context state");
    }

    fn draw_placeholder(&self, cr: &Context, width: i32, height: i32) {
        cr.save().expect("Failed to save context state");
        
        cr.set_source_rgb(0.3, 0.3, 0.3);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(20.0);

        let text = "Open or create a document to start editing";
        let extents = cr.text_extents(text).expect("Failed to get text extents");
        let x = (width as f64 - extents.width()) / 2.0;
        let y = (height as f64 + extents.height()) / 2.0;

        cr.move_to(x, y);
        cr.show_text(text).expect("Failed to show text");
        
        cr.restore().expect("Failed to restore context state");
    }

    fn on_zoom(&mut self, scale: f64) {
        self.scale = scale.max(0.1).min(10.0);
        self.drawing_area.queue_draw();
    }

    fn on_drag_begin(&mut self, x: f64, y: f64) {
        self.offset_x = x;
        self.offset_y = y;
    }

    fn on_drag_update(&mut self, x: f64, y: f64) {
        self.offset_x += x;
        self.offset_y += y;
        self.drawing_area.queue_draw();
    }

    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    pub fn set_tool_manager(&mut self, tool_manager: Option<ToolManager>) {
        info!("Setting tool manager in canvas");
        self.tool_manager = tool_manager;
    }

    pub fn zoom_to_fit(&mut self) {
        info!("Zooming to fit");
        
        // Get the widget dimensions
        let width = self.drawing_area.width();
        let height = self.drawing_area.height();
        
        // Calculate scale to fit
        let scale_x = width as f64 / self.width as f64;
        let scale_y = height as f64 / self.height as f64;
        self.scale = scale_x.min(scale_y) * 0.9; // 90% to leave a margin
        
        // Center the canvas
        self.center();
        
        // Redraw
        self.drawing_area.queue_draw();
    }
    
    pub fn zoom_to_actual(&mut self) {
        info!("Zooming to actual size (100%)");
        
        // Reset scale
        self.scale = 1.0;
        
        // Center the canvas
        self.center();
        
        // Redraw
        self.drawing_area.queue_draw();
    }
    
    pub fn center(&mut self) {
        // Get the widget dimensions
        let width = self.drawing_area.width();
        let height = self.drawing_area.height();
        
        // Calculate center position
        self.offset_x = (width as f64 - self.width as f64 * self.scale) / 2.0;
        self.offset_y = (height as f64 - self.height as f64 * self.scale) / 2.0;
    }
}

impl Clone for Canvas {
    fn clone(&self) -> Self {
        Self {
            drawing_area: self.drawing_area.clone(),
            document: self.document.clone(),
            layer_manager: self.layer_manager.clone(),
            scale: self.scale,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            width: self.width,
            height: self.height,
            tool_manager: self.tool_manager.clone(),
            vector_document: self.vector_document.clone(),
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
} 