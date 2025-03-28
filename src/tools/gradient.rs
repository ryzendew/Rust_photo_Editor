use crate::core::Canvas;
use crate::vector::Point;
use super::ToolImpl;
use image::{Rgba, ImageBuffer};
use crate::core::{Layer};
use crate::tools::{Tool, ToolType};
use cairo::Context;

#[derive(Clone)]
pub enum GradientType {
    Linear,
    Radial,
    Angular,
    Diamond,
    Reflected,
}

#[derive(Clone)]
pub struct GradientTool {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub is_dragging: bool,
    pub color1: Rgba<u8>,
    pub color2: Rgba<u8>,
    pub gradient_type: GradientType,
    pub active: bool,
}

impl GradientTool {
    pub fn new() -> Self {
        Self {
            start_point: None,
            end_point: None,
            is_dragging: false,
            color1: Rgba([0, 0, 0, 255]),
            color2: Rgba([255, 255, 255, 255]),
            gradient_type: GradientType::Linear,
            active: false,
        }
    }
    
    pub fn set_colors(&mut self, color1: Rgba<u8>, color2: Rgba<u8>) {
        self.color1 = color1;
        self.color2 = color2;
    }
    
    pub fn set_gradient_type(&mut self, gradient_type: GradientType) {
        self.gradient_type = gradient_type;
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.start_point = None;
            self.end_point = None;
            self.is_dragging = false;
        }
    }
    
    fn apply_gradient(&self, canvas: &mut Canvas) -> bool {
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            if let Some(layer) = canvas.layer_manager.get_active_layer_mut() {
                let image = &mut layer.image;
                let width = image.width();
                let height = image.height();
                
                // Create a new buffer for the gradient
                let mut gradient_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = 
                    ImageBuffer::new(width, height);
                
                match self.gradient_type {
                    GradientType::Linear => {
                        self.apply_linear_gradient(&mut gradient_buffer, start, end);
                    },
                    GradientType::Radial => {
                        self.apply_radial_gradient(&mut gradient_buffer, start, end);
                    },
                    GradientType::Angular => {
                        self.apply_angular_gradient(&mut gradient_buffer, start, end);
                    },
                    GradientType::Diamond => {
                        self.apply_diamond_gradient(&mut gradient_buffer, start, end);
                    },
                    GradientType::Reflected => {
                        self.apply_reflected_gradient(&mut gradient_buffer, start, end);
                    }
                }
                
                // Copy the gradient to the layer image
                *image = gradient_buffer;
                
                return true;
            }
        }
        
        false
    }
    
    fn apply_linear_gradient(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, 
                           start: Point, end: Point) {
        let width = buffer.width();
        let height = buffer.height();
        
        // Calculate vector from start to end
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length_squared = dx * dx + dy * dy;
        
        for y in 0..height {
            for x in 0..width {
                // Calculate projection of current point onto gradient line
                let px = x as f64 - start.x;
                let py = y as f64 - start.y;
                
                // Dot product divided by length squared gives normalized position along gradient
                let mut t = (px * dx + py * dy) / length_squared;
                
                // Clamp t to [0, 1]
                t = t.max(0.0).min(1.0);
                
                // Interpolate colors
                let pixel = self.interpolate_colors(t);
                buffer.put_pixel(x, y, pixel);
            }
        }
    }
    
    fn apply_radial_gradient(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, 
                           center: Point, outer: Point) {
        let width = buffer.width();
        let height = buffer.height();
        
        // Calculate radius
        let radius = center.distance_to(&outer);
        
        for y in 0..height {
            for x in 0..width {
                // Calculate distance from center
                let dist = Point::new(x as f64, y as f64).distance_to(&center);
                
                // Normalize distance by radius
                let mut t = dist / radius;
                
                // Clamp t to [0, 1]
                t = t.max(0.0).min(1.0);
                
                // Interpolate colors
                let pixel = self.interpolate_colors(t);
                buffer.put_pixel(x, y, pixel);
            }
        }
    }
    
    fn apply_angular_gradient(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, 
                            center: Point, point: Point) {
        let width = buffer.width();
        let height = buffer.height();
        
        // Calculate the angle of the reference point
        let ref_angle = (point.y - center.y).atan2(point.x - center.x);
        
        for y in 0..height {
            for x in 0..width {
                // Calculate angle of current point relative to center
                let angle = (y as f64 - center.y).atan2(x as f64 - center.x);
                
                // Normalize angle difference to [0, 1]
                let mut t = ((angle - ref_angle) / (2.0 * std::f64::consts::PI) + 1.0) % 1.0;
                
                // Clamp t to [0, 1] (redundant but just to be safe)
                t = t.max(0.0).min(1.0);
                
                // Interpolate colors
                let pixel = self.interpolate_colors(t);
                buffer.put_pixel(x, y, pixel);
            }
        }
    }
    
    fn apply_diamond_gradient(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, 
                            center: Point, point: Point) {
        let width = buffer.width();
        let height = buffer.height();
        
        // Calculate radius as Manhattan distance
        let radius = (point.x - center.x).abs() + (point.y - center.y).abs();
        
        for y in 0..height {
            for x in 0..width {
                // Calculate Manhattan distance from center
                let dist = (x as f64 - center.x).abs() + (y as f64 - center.y).abs();
                
                // Normalize distance by radius
                let mut t = dist / radius;
                
                // Clamp t to [0, 1]
                t = t.max(0.0).min(1.0);
                
                // Interpolate colors
                let pixel = self.interpolate_colors(t);
                buffer.put_pixel(x, y, pixel);
            }
        }
    }
    
    fn apply_reflected_gradient(&self, buffer: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, 
                              start: Point, end: Point) {
        let width = buffer.width();
        let height = buffer.height();
        
        // Calculate vector from start to end
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length_squared = dx * dx + dy * dy;
        
        for y in 0..height {
            for x in 0..width {
                // Calculate projection of current point onto gradient line
                let px = x as f64 - start.x;
                let py = y as f64 - start.y;
                
                // Dot product divided by length squared gives normalized position along gradient
                let mut t = (px * dx + py * dy) / length_squared;
                
                // For reflected gradient, convert t to range [0, 1] by reflecting at 0.5
                t = if t < 0.0 {
                    (-t).max(0.0).min(1.0)
                } else if t > 1.0 {
                    (2.0 - t).max(0.0).min(1.0)
                } else if t < 0.5 {
                    t * 2.0
                } else {
                    (1.0 - t) * 2.0
                };
                
                // Interpolate colors
                let pixel = self.interpolate_colors(t);
                buffer.put_pixel(x, y, pixel);
            }
        }
    }
    
    fn interpolate_colors(&self, t: f64) -> Rgba<u8> {
        // Linear interpolation between color1 and color2 using t
        let r = ((1.0 - t) * self.color1.0[0] as f64 + t * self.color2.0[0] as f64) as u8;
        let g = ((1.0 - t) * self.color1.0[1] as f64 + t * self.color2.0[1] as f64) as u8;
        let b = ((1.0 - t) * self.color1.0[2] as f64 + t * self.color2.0[2] as f64) as u8;
        let a = ((1.0 - t) * self.color1.0[3] as f64 + t * self.color2.0[3] as f64) as u8;
        
        Rgba([r, g, b, a])
    }
}

impl ToolImpl for GradientTool {
    fn on_mouse_down(&mut self, _canvas: &mut Canvas, x: f64, y: f64) -> bool {
        self.start_point = Some(Point::new(x, y));
        self.end_point = Some(Point::new(x, y));
        self.is_dragging = true;
        true
    }
    
    fn on_mouse_drag(&mut self, _canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if self.is_dragging {
            self.end_point = Some(Point::new(x, y));
            return true;
        }
        false
    }
    
    fn on_mouse_up(&mut self, canvas: &mut Canvas, _x: f64, _y: f64) -> bool {
        if self.is_dragging {
            self.is_dragging = false;
            
            // Apply the gradient to the active layer
            return self.apply_gradient(canvas);
        }
        false
    }
    
    fn get_cursor(&self) -> Option<String> {
        Some("crosshair".to_string())
    }
    
    fn draw(&self, _canvas: &Canvas, context: &cairo::Context) {
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            // Draw a line showing the gradient direction
            context.set_source_rgba(1.0, 1.0, 1.0, 0.8);
            context.set_line_width(2.0);
            context.move_to(start.x, start.y);
            context.line_to(end.x, end.y);
            context.stroke();
            
            // Draw small circles at start and end points
            let radius = 5.0;
            
            // Start point - color1
            context.set_source_rgba(
                self.color1.0[0] as f64 / 255.0,
                self.color1.0[1] as f64 / 255.0,
                self.color1.0[2] as f64 / 255.0,
                self.color1.0[3] as f64 / 255.0
            );
            context.arc(start.x, start.y, radius, 0.0, 2.0 * std::f64::consts::PI);
            context.fill();
            
            // End point - color2
            context.set_source_rgba(
                self.color2.0[0] as f64 / 255.0,
                self.color2.0[1] as f64 / 255.0,
                self.color2.0[2] as f64 / 255.0,
                self.color2.0[3] as f64 / 255.0
            );
            context.arc(end.x, end.y, radius, 0.0, 2.0 * std::f64::consts::PI);
            context.fill();
        }
    }
}

impl Tool for GradientTool {
    fn tool_type(&self) -> ToolType {
        ToolType::Gradient
    }
    
    fn cursor(&self) -> &'static str {
        "crosshair"
    }
    
    fn active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.start_point = None;
            self.end_point = None;
            self.is_dragging = false;
        }
    }
    
    fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        if button != 1 || !self.active {
            return;
        }
        
        self.start_point = Some(Point::new(x, y));
        self.end_point = Some(Point::new(x, y));
        self.is_dragging = true;
    }
    
    fn mouse_move(&mut self, x: f64, y: f64) {
        if !self.active || !self.is_dragging {
            return;
        }
        
        self.end_point = Some(Point::new(x, y));
    }
    
    fn mouse_up(&mut self, x: f64, y: f64, button: u32) {
        if button != 1 || !self.active || !self.is_dragging {
            return;
        }
        
        self.end_point = Some(Point::new(x, y));
        self.is_dragging = false;
        
        // Note: Actual gradient application would happen through ToolImpl.on_mouse_up
        // with access to the canvas, which isn't available here
    }
    
    fn key_press(&mut self, key: &str) {
        // Handle key press for gradient tool
        match key {
            "Escape" => {
                self.start_point = None;
                self.end_point = None;
                self.is_dragging = false;
            },
            _ => {}
        }
    }
    
    fn draw_preview(&self, context: &cairo::Context, canvas: &Canvas) {
        if !self.active {
            return;
        }
        
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            context.save();
            
            // Draw a line showing the gradient direction
            context.set_source_rgba(1.0, 1.0, 1.0, 0.8);
            context.set_line_width(2.0);
            context.move_to(start.x, start.y);
            context.line_to(end.x, end.y);
            context.stroke();
            
            // Draw small circles at start and end points
            let radius = 5.0;
            
            // Start point - color1
            context.set_source_rgba(
                self.color1.0[0] as f64 / 255.0,
                self.color1.0[1] as f64 / 255.0,
                self.color1.0[2] as f64 / 255.0,
                self.color1.0[3] as f64 / 255.0
            );
            context.arc(start.x, start.y, radius, 0.0, 2.0 * std::f64::consts::PI);
            context.fill();
            
            // End point - color2
            context.set_source_rgba(
                self.color2.0[0] as f64 / 255.0,
                self.color2.0[1] as f64 / 255.0,
                self.color2.0[2] as f64 / 255.0,
                self.color2.0[3] as f64 / 255.0
            );
            context.arc(end.x, end.y, radius, 0.0, 2.0 * std::f64::consts::PI);
            context.fill();
            
            // Add a label showing the current gradient type
            context.set_source_rgba(1.0, 1.0, 1.0, 0.9);
            context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            context.set_font_size(12.0);
            
            let label = match self.gradient_type {
                GradientType::Linear => "Linear",
                GradientType::Radial => "Radial",
                GradientType::Angular => "Angular", 
                GradientType::Diamond => "Diamond",
                GradientType::Reflected => "Reflected"
            };
            
            let mid_x = (start.x + end.x) / 2.0;
            let mid_y = (start.y + end.y) / 2.0 - 15.0; // Position above the line
            
            // Draw a background for the text for better visibility
            let text_extents = context.text_extents(label).unwrap();
            context.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            context.rectangle(
                mid_x - text_extents.width() / 2.0 - 3.0,
                mid_y - text_extents.height() - 3.0,
                text_extents.width() + 6.0,
                text_extents.height() + 6.0
            );
            context.fill();
            
            // Draw the label text
            context.set_source_rgba(1.0, 1.0, 1.0, 0.9);
            context.move_to(
                mid_x - text_extents.width() / 2.0,
                mid_y
            );
            context.show_text(label).unwrap();
            
            context.restore();
        }
    }
} 