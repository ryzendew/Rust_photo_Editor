use crate::core::{Canvas, Layer};
use crate::vector::Point;
use image::{ImageBuffer, Rgba};
use super::ToolImpl;
use crate::tools::{Tool, ToolType};
use cairo::Context;

#[derive(Clone)]
pub struct HealSettings {
    pub radius: f64,
    pub hardness: f64,
    pub tolerance: f64,
}

#[derive(Clone)]
pub struct HealTool {
    pub active: bool,
    pub settings: HealSettings,
    pub source_point: Option<Point>,
    pub destination_point: Option<Point>,
    pub last_point: Option<Point>,
}

impl HealTool {
    pub fn new() -> Self {
        Self {
            active: false,
            settings: HealSettings {
                radius: 10.0,
                hardness: 0.5,
                tolerance: 0.3,
            },
            source_point: None,
            destination_point: None,
            last_point: None,
        }
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.source_point = None;
            self.destination_point = None;
            self.last_point = None;
        }
    }
    
    pub fn cursor(&self) -> &'static str {
        if self.source_point.is_some() {
            "copy"
        } else {
            "crosshair"
        }
    }
}

impl Tool for HealTool {
    fn tool_type(&self) -> ToolType {
        ToolType::Heal
    }
    
    fn cursor(&self) -> &'static str {
        if self.source_point.is_some() {
            "copy"
        } else {
            "crosshair"
        }
    }
    
    fn active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.source_point = None;
            self.destination_point = None;
            self.last_point = None;
        }
    }
    
    fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        if button != 1 || !self.active {
            return;
        }
        
        // Alt key is often used to define source point (would need event state in real impl)
        if self.source_point.is_none() {
            self.source_point = Some(Point::new(x, y));
        } else {
            self.destination_point = Some(Point::new(x, y));
        }
        
        self.last_point = Some(Point::new(x, y));
    }
    
    fn mouse_move(&mut self, x: f64, y: f64) {
        if !self.active {
            return;
        }
        
        self.last_point = Some(Point::new(x, y));
    }
    
    fn mouse_up(&mut self, _x: f64, _y: f64, button: u32) {
        if button != 1 || !self.active {
            return;
        }
        
        // We keep the last point for preview/cursor display
    }
    
    fn key_press(&mut self, key: &str) {
        // Handle key presses specific to the heal tool
        match key {
            "Escape" => {
                self.source_point = None;
                self.destination_point = None;
                self.last_point = None;
            },
            "Alt" => {
                // Alt key might be used to pick source point
                if let Some(last) = self.last_point {
                    self.source_point = Some(last);
                }
            }
            _ => {}
        }
    }
    
    fn draw_preview(&self, context: &Context, _canvas: &Canvas) {
        // Draw a preview of the healing brush
        if let Some(last) = self.last_point {
            context.save();
            
            // Draw a circle representing the brush size
            context.set_source_rgba(0.3, 0.6, 1.0, 0.5);
            context.set_line_width(1.0);
            context.arc(last.x, last.y, self.settings.radius, 0.0, 2.0 * std::f64::consts::PI);
            context.stroke();
            
            // Draw inner circle showing the hardness falloff
            let inner_radius = self.settings.radius * (1.0 - self.settings.hardness);
            if inner_radius > 0.0 {
                context.set_source_rgba(0.3, 0.6, 1.0, 0.2);
                context.arc(last.x, last.y, inner_radius, 0.0, 2.0 * std::f64::consts::PI);
                context.stroke();
            }
            
            // If we have a source point, draw a line connecting them
            if let Some(source) = self.source_point {
                // Draw a dashed line between source and destination
                context.set_source_rgba(0.3, 0.6, 1.0, 0.4);
                context.set_line_width(1.0);
                context.set_dash(&[5.0, 5.0], 0.0);
                context.move_to(source.x, source.y);
                context.line_to(last.x, last.y);
                context.stroke();
                
                // Draw a crosshair at the source point
                context.set_source_rgba(0.1, 0.8, 0.1, 0.7);
                context.set_line_width(1.5);
                context.set_dash(&[], 0.0);  // Clear dash pattern
                
                // Horizontal line of crosshair
                context.move_to(source.x - 6.0, source.y);
                context.line_to(source.x + 6.0, source.y);
                context.stroke();
                
                // Vertical line of crosshair
                context.move_to(source.x, source.y - 6.0);
                context.line_to(source.x, source.y + 6.0);
                context.stroke();
                
                // Draw a circle at the source point
                context.arc(source.x, source.y, 3.0, 0.0, 2.0 * std::f64::consts::PI);
                context.stroke();
            }
            
            context.restore();
        }
    }
}

impl ToolImpl for HealTool {
    fn on_mouse_down(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        // Similar to clone tool, but we will blend the textures instead of just copying
        if self.source_point.is_none() {
            self.source_point = Some(Point::new(x, y));
            return true;
        }
        
        self.destination_point = Some(Point::new(x, y));
        self.last_point = Some(Point::new(x, y));
        
        // Apply the healing
        self.heal_pixels(canvas, x, y);
        
        true
    }
    
    fn on_mouse_drag(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if self.source_point.is_none() || self.destination_point.is_none() {
            return false;
        }
        
        if let Some(last) = self.last_point {
            let curr = Point::new(x, y);
            let dist = last.distance_to(&curr);
            let step_size = self.settings.radius / 4.0;
            
            if dist > 0.0 {
                let steps = (dist / step_size).ceil() as usize;
                
                for i in 0..=steps {
                    let t = if steps == 0 { 0.0 } else { i as f64 / steps as f64 };
                    let ix = last.x + (curr.x - last.x) * t;
                    let iy = last.y + (curr.y - last.y) * t;
                    
                    self.heal_pixels(canvas, ix, iy);
                }
            }
        }
        
        self.last_point = Some(Point::new(x, y));
        true
    }
    
    fn on_mouse_up(&mut self, _canvas: &mut Canvas, _x: f64, _y: f64) -> bool {
        self.last_point = None;
        true
    }
    
    fn get_cursor(&self) -> Option<String> {
        Some("heal".to_string())
    }
}

impl HealTool {
    fn heal_pixels(&self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if let (Some(source), Some(dest)) = (self.source_point, self.destination_point) {
            // Calculate offset between source and destination
            let offset_x = source.x - dest.x;
            let offset_y = source.y - dest.y;
            
            // Calculate source position corresponding to current position
            let src_x = x + offset_x;
            let src_y = y + offset_y;
            
            if let Some(layer) = canvas.layer_manager.get_active_layer_mut() {
                let image = &mut layer.image;
                let size = self.settings.radius as i32;
                let cx = x as i32;
                let cy = y as i32;
                
                let width = image.width() as i32;
                let height = image.height() as i32;
                
                // Calculate average color from source region
                let mut avg_r = 0.0;
                let mut avg_g = 0.0;
                let mut avg_b = 0.0;
                let mut avg_a = 0.0;
                let mut count = 0;
                
                // Get average color from source
                for dy in -size..=size {
                    for dx in -size..=size {
                        let sx = (src_x as i32) + dx;
                        let sy = (src_y as i32) + dy;
                        
                        if sx >= 0 && sx < width && sy >= 0 && sy < height {
                            let dist = ((dx*dx + dy*dy) as f64).sqrt();
                            if dist <= self.settings.radius {
                                let pixel = image.get_pixel(sx as u32, sy as u32);
                                avg_r += pixel.0[0] as f64;
                                avg_g += pixel.0[1] as f64;
                                avg_b += pixel.0[2] as f64;
                                avg_a += pixel.0[3] as f64;
                                count += 1;
                            }
                        }
                    }
                }
                
                if count > 0 {
                    avg_r /= count as f64;
                    avg_g /= count as f64;
                    avg_b /= count as f64;
                    avg_a /= count as f64;
                    
                    // Apply the healing by blending source texture with destination colors
                    for dy in -size..=size {
                        for dx in -size..=size {
                            let px = cx + dx;
                            let py = cy + dy;
                            
                            // Source pixel
                            let sx = (src_x as i32) + dx;
                            let sy = (src_y as i32) + dy;
                            
                            // Check if in bounds
                            if px >= 0 && px < width && py >= 0 && py < height &&
                               sx >= 0 && sx < width && sy >= 0 && sy < height {
                                // Calculate distance from center
                                let dist = ((dx*dx + dy*dy) as f64).sqrt();
                                if dist <= self.settings.radius {
                                    // Calculate opacity based on distance and hardness
                                    let alpha = if dist < self.settings.radius * (1.0 - self.settings.hardness) {
                                        1.0
                                    } else {
                                        let t = (self.settings.radius - dist) / (self.settings.radius * self.settings.hardness);
                                        t
                                    };
                                    
                                    if alpha > 0.0 {
                                        // Get source pixel
                                        let src_pixel = image.get_pixel(sx as u32, sy as u32);
                                        // Get destination pixel
                                        let dst_pixel = image.get_pixel(px as u32, py as u32);
                                        
                                        // Calculate color differences
                                        let src_r = src_pixel.0[0] as f64;
                                        let src_g = src_pixel.0[1] as f64;
                                        let src_b = src_pixel.0[2] as f64;
                                        let src_a = src_pixel.0[3] as f64;
                                        
                                        // Determine texture variation from average
                                        let diff_r = src_r - avg_r;
                                        let diff_g = src_g - avg_g;
                                        let diff_b = src_b - avg_b;
                                        let diff_a = src_a - avg_a;
                                        
                                        // Apply texture to destination
                                        let new_r = (dst_pixel.0[0] as f64 + diff_r).max(0.0).min(255.0);
                                        let new_g = (dst_pixel.0[1] as f64 + diff_g).max(0.0).min(255.0);
                                        let new_b = (dst_pixel.0[2] as f64 + diff_b).max(0.0).min(255.0);
                                        let new_a = (dst_pixel.0[3] as f64 + diff_a).max(0.0).min(255.0);
                                        
                                        // Blend with existing pixel based on alpha
                                        let mut rgba = dst_pixel.0;
                                        rgba[0] = ((1.0 - alpha) * rgba[0] as f64 + alpha * new_r) as u8;
                                        rgba[1] = ((1.0 - alpha) * rgba[1] as f64 + alpha * new_g) as u8;
                                        rgba[2] = ((1.0 - alpha) * rgba[2] as f64 + alpha * new_b) as u8;
                                        rgba[3] = ((1.0 - alpha) * rgba[3] as f64 + alpha * new_a) as u8;
                                        
                                        image.put_pixel(px as u32, py as u32, image::Rgba(rgba));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return true;
        }
        false
    }
} 