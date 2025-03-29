use crate::core::Canvas;
use crate::vector::{Point, Rect};
use super::ToolImpl;
use crate::tools::{Tool, ToolType};
use cairo::Context;

#[derive(Clone)]
pub struct CropTool {
    pub active: bool,
    pub start_x: Option<f64>,
    pub start_y: Option<f64>,
    pub end_x: Option<f64>,
    pub end_y: Option<f64>,
    pub dragging: bool,
    pub handle_index: Option<usize>,
}

impl CropTool {
    pub fn new() -> Self {
        Self {
            active: false,
            start_x: None,
            start_y: None,
            end_x: None,
            end_y: None,
            dragging: false,
            handle_index: None,
        }
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.clear_selection();
        }
    }
    
    pub fn clear_selection(&mut self) {
        self.start_x = None;
        self.start_y = None;
        self.end_x = None;
        self.end_y = None;
        self.dragging = false;
        self.handle_index = None;
    }
    
    pub fn cursor(&self) -> &'static str {
        if self.dragging {
            "grabbing"
        } else if self.handle_index.is_some() {
            match self.handle_index.unwrap() {
                0 | 4 => "nwse-resize", // top-left or bottom-right
                1 | 5 => "ns-resize",   // top-center or bottom-center
                2 | 6 => "nesw-resize", // top-right or bottom-left
                3 | 7 => "ew-resize",   // middle-right or middle-left
                _ => "crosshair",
            }
        } else if self.start_x.is_some() && self.end_x.is_some() {
            "move" // When hovering over the selection
        } else {
            "crosshair"
        }
    }
    
    pub fn get_rectangle(&self) -> Option<(f64, f64, f64, f64)> {
        match (self.start_x, self.start_y, self.end_x, self.end_y) {
            (Some(sx), Some(sy), Some(ex), Some(ey)) => {
                let x = sx.min(ex);
                let y = sy.min(ey);
                let width = (ex - sx).abs();
                let height = (ey - sy).abs();
                
                Some((x, y, width, height))
            },
            _ => None
        }
    }
    
    pub fn is_complete(&self) -> bool {
        if let Some((_, _, width, height)) = self.get_rectangle() {
            // Consider a crop complete if it has a non-zero area
            return width > 1.0 && height > 1.0;
        }
        false
    }
    
    pub fn get_crop_rect(&self) -> Option<Rect> {
        self.get_rectangle().map(|(x, y, width, height)| {
            Rect::new(x, y, width, height)
        })
    }
    
    pub fn reset(&mut self) {
        self.clear_selection();
    }
}

impl Tool for CropTool {
    fn tool_type(&self) -> ToolType {
        ToolType::Crop
    }
    
    fn cursor(&self) -> &'static str {
        self.cursor()
    }
    
    fn active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.set_active(active)
    }
    
    fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        // This would typically call on_mouse_down with a canvas
        if button == 1 { // left button
            if self.start_x.is_none() {
                self.start_x = Some(x);
                self.start_y = Some(y);
                self.end_x = Some(x);
                self.end_y = Some(y);
                self.dragging = true;
            } else if let Some(handle) = self.handle_index {
                // Resizing with a handle
                self.dragging = true;
            } else if let Some((rect_x, rect_y, rect_w, rect_h)) = self.get_rectangle() {
                // Check if the click is inside the crop rectangle
                if x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h {
                    self.dragging = true;
                } else {
                    // Start a new selection
                    self.start_x = Some(x);
                    self.start_y = Some(y);
                    self.end_x = Some(x);
                    self.end_y = Some(y);
                    self.dragging = true;
                }
            }
        }
    }
    
    fn mouse_move(&mut self, x: f64, y: f64) {
        // This would typically call on_mouse_drag with a canvas
        if self.dragging {
            self.end_x = Some(x);
            self.end_y = Some(y);
        } else {
            // Check if mouse is over a handle
            if let Some((rect_x, rect_y, rect_w, rect_h)) = self.get_rectangle() {
                let handle_size = 10.0;
                let half_handle = handle_size / 2.0;
                
                let handles = [
                    (rect_x, rect_y),                   // 0: top-left
                    (rect_x + rect_w/2.0, rect_y),      // 1: top-center
                    (rect_x + rect_w, rect_y),          // 2: top-right
                    (rect_x + rect_w, rect_y + rect_h/2.0), // 3: middle-right
                    (rect_x + rect_w, rect_y + rect_h), // 4: bottom-right
                    (rect_x + rect_w/2.0, rect_y + rect_h), // 5: bottom-center
                    (rect_x, rect_y + rect_h),          // 6: bottom-left
                    (rect_x, rect_y + rect_h/2.0),      // 7: middle-left
                ];
                
                self.handle_index = None;
                for (i, (hx, hy)) in handles.iter().enumerate() {
                    if x >= hx - half_handle && x <= hx + half_handle && 
                       y >= hy - half_handle && y <= hy + half_handle {
                        self.handle_index = Some(i);
                        break;
                    }
                }
            }
        }
    }
    
    fn mouse_up(&mut self, _x: f64, _y: f64, button: u32) {
        // This would typically call on_mouse_up with a canvas
        if button == 1 {
            self.dragging = false;
        }
    }
    
    fn key_press(&mut self, key: &str) {
        // Handle key presses specific to the crop tool
        match key {
            "Escape" => {
                self.clear_selection();
            },
            "Return" => {
                // Apply the crop (would need canvas access)
                // After applying, reset the selection
                self.clear_selection();
            },
            _ => {}
        }
    }
    
    fn draw_preview(&self, context: &Context, canvas: &Canvas) {
        // Draw the crop rectangle and handles
        if let Some((x, y, width, height)) = self.get_rectangle() {
            context.save();
            
            // Draw darkened overlay
            context.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            context.set_operator(cairo::Operator::Over);
            
            // Draw the entire canvas
            let canvas_width = canvas.width as f64;
            let canvas_height = canvas.height as f64;
            
            // Top
            context.rectangle(0.0, 0.0, canvas_width, y);
            context.fill();
            
            // Left
            context.rectangle(0.0, y, x, height);
            context.fill();
            
            // Right
            context.rectangle(x + width, y, canvas_width - (x + width), height);
            context.fill();
            
            // Bottom
            context.rectangle(0.0, y + height, canvas_width, canvas_height - (y + height));
            context.fill();
            
            // Draw crop rectangle border
            context.set_source_rgba(1.0, 1.0, 1.0, 0.8);
            context.set_line_width(2.0);
            context.rectangle(x, y, width, height);
            context.stroke();
            
            // Draw grid lines (rule of thirds)
            context.set_source_rgba(1.0, 1.0, 1.0, 0.5);
            context.set_line_width(1.0);
            context.set_dash(&[5.0, 5.0], 0.0);
            
            // Vertical grid lines
            let third_w = width / 3.0;
            context.move_to(x + third_w, y);
            context.line_to(x + third_w, y + height);
            context.move_to(x + third_w * 2.0, y);
            context.line_to(x + third_w * 2.0, y + height);
            
            // Horizontal grid lines
            let third_h = height / 3.0;
            context.move_to(x, y + third_h);
            context.line_to(x + width, y + third_h);
            context.move_to(x, y + third_h * 2.0);
            context.line_to(x + width, y + third_h * 2.0);
            
            context.stroke();
            
            // Draw handles
            context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            context.set_dash(&[], 0.0);
            let handle_size = 8.0;
            
            let handles = [
                (x, y),                     // top-left
                (x + width/2.0, y),         // top-center
                (x + width, y),             // top-right
                (x + width, y + height/2.0), // middle-right
                (x + width, y + height),    // bottom-right
                (x + width/2.0, y + height), // bottom-center
                (x, y + height),            // bottom-left
                (x, y + height/2.0),        // middle-left
            ];
            
            for (hx, hy) in handles {
                context.rectangle(
                    hx - handle_size/2.0, 
                    hy - handle_size/2.0, 
                    handle_size, 
                    handle_size
                );
                context.fill();
            }
            
            context.restore();
        }
    }
}

impl ToolImpl for CropTool {
    fn on_mouse_down(&mut self, _canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if self.start_x.is_none() {
            self.start_x = Some(x);
            self.start_y = Some(y);
            self.end_x = Some(x);
            self.end_y = Some(y);
            self.dragging = true;
            return true;
        }
        
        false
    }
    
    fn on_mouse_drag(&mut self, _canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if self.dragging {
            self.end_x = Some(x);
            self.end_y = Some(y);
            return true;
        }
        
        false
    }
    
    fn on_mouse_up(&mut self, _canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if self.dragging {
            self.end_x = Some(x);
            self.end_y = Some(y);
            self.dragging = false;
            return true;
        }
        
        false
    }
    
    fn draw(&self, canvas: &Canvas, context: &cairo::Context) {
        if let Some(rect) = self.get_rectangle() {
            // Draw the crop rectangle
            context.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            
            // Draw the four regions outside the crop rectangle
            // Top
            context.rectangle(0.0, 0.0, canvas.width as f64, rect.1);
            // Left
            context.rectangle(0.0, rect.1, rect.0, rect.3);
            // Right
            context.rectangle(rect.0 + rect.2, rect.1, 
                            canvas.width as f64 - (rect.0 + rect.2), rect.3);
            // Bottom
            context.rectangle(0.0, rect.1 + rect.3, 
                            canvas.width as f64, canvas.height as f64 - (rect.1 + rect.3));
            context.fill();
            
            // Draw border of crop rectangle
            context.set_source_rgba(1.0, 1.0, 1.0, 0.8);
            context.set_line_width(2.0);
            context.rectangle(rect.0, rect.1, rect.2, rect.3);
            context.stroke();
            
            // Draw rule-of-thirds grid
            context.set_source_rgba(1.0, 1.0, 1.0, 0.5);
            context.set_line_width(1.0);
            context.set_dash(&[5.0, 5.0], 0.0);
            
            // Vertical lines
            for i in 1..=2 {
                let x = rect.0 + rect.2 * (i as f64 / 3.0);
                context.move_to(x, rect.1);
                context.line_to(x, rect.1 + rect.3);
                context.stroke();
            }
            
            // Horizontal lines
            for i in 1..=2 {
                let y = rect.1 + rect.3 * (i as f64 / 3.0);
                context.move_to(rect.0, y);
                context.line_to(rect.0 + rect.2, y);
                context.stroke();
            }
            
            // Draw handles
            context.set_dash(&[], 0.0);  // Remove dash pattern
            
            // Draw handles at corners and midpoints of edges
            let handle_size = 5.0;
            
            context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            context.set_line_width(2.0);
            
            // Top-left
            context.rectangle(rect.0 - handle_size, rect.1 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Top-right
            context.rectangle(rect.0 + rect.2 - handle_size, rect.1 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Bottom-left
            context.rectangle(rect.0 - handle_size, rect.1 + rect.3 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Bottom-right
            context.rectangle(rect.0 + rect.2 - handle_size, rect.1 + rect.3 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Top-middle
            context.rectangle(rect.0 + rect.2 / 2.0 - handle_size, rect.1 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Right-middle
            context.rectangle(rect.0 + rect.2 - handle_size, rect.1 + rect.3 / 2.0 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Bottom-middle
            context.rectangle(rect.0 + rect.2 / 2.0 - handle_size, rect.1 + rect.3 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
            
            // Left-middle
            context.rectangle(rect.0 - handle_size, rect.1 + rect.3 / 2.0 - handle_size, 
                            handle_size * 2.0, handle_size * 2.0);
            context.stroke();
        }
    }
} 