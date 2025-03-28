use crate::core::Canvas;
use crate::vector::Point;
use super::ToolImpl;

#[derive(Clone)]
pub struct CloneTool {
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
    pub source_point: Option<Point>,
    pub destination_point: Option<Point>,
    pub last_point: Option<Point>,
    pub active: bool,
}

impl CloneTool {
    pub fn new() -> Self {
        Self {
            size: 20.0,
            hardness: 0.5,
            opacity: 1.0,
            source_point: None,
            destination_point: None,
            last_point: None,
            active: false,
        }
    }
    
    pub fn set_source(&mut self, x: f64, y: f64) {
        self.source_point = Some(Point::new(x, y));
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.last_point = None;
            // Note: We don't clear source_point when deactivating
            // as we might want to remember the clone source
        }
    }
}

impl ToolImpl for CloneTool {
    fn on_mouse_down(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        // If Alt key is pressed (would be handled through event flags in real implementation)
        // set the source point instead of cloning
        // For now, we'll assume first click sets source, second click starts cloning
        
        if self.source_point.is_none() {
            self.set_source(x, y);
            return true;
        }
        
        self.destination_point = Some(Point::new(x, y));
        self.last_point = Some(Point::new(x, y));
        
        // Actually clone the pixels
        self.clone_pixels(canvas, x, y);
        
        true
    }
    
    fn on_mouse_drag(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if self.source_point.is_none() || self.destination_point.is_none() {
            return false;
        }
        
        if let Some(last) = self.last_point {
            // Draw a line from last point to current point
            // Similar to brush tool
            let curr = Point::new(x, y);
            let dist = last.distance_to(&curr);
            let step_size = self.size / 4.0;
            
            if dist > 0.0 {
                let steps = (dist / step_size).ceil() as usize;
                
                for i in 0..=steps {
                    let t = if steps == 0 { 0.0 } else { i as f64 / steps as f64 };
                    let ix = last.x + (curr.x - last.x) * t;
                    let iy = last.y + (curr.y - last.y) * t;
                    
                    self.clone_pixels(canvas, ix, iy);
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
        Some("clone".to_string())
    }
}

impl CloneTool {
    fn clone_pixels(&self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if let (Some(source), Some(dest)) = (self.source_point, self.destination_point) {
            // Calculate offset between source and destination
            let offset_x = source.x - dest.x;
            let offset_y = source.y - dest.y;
            
            // Calculate source position corresponding to current position
            let src_x = x + offset_x;
            let src_y = y + offset_y;
            
            if let Some(layer) = canvas.layer_manager.get_active_layer_mut() {
                let image = &mut layer.image;
                let size = self.size as i32;
                let cx = x as i32;
                let cy = y as i32;
                
                let width = image.width() as i32;
                let height = image.height() as i32;
                
                // Simple circle cloning
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
                            if dist <= self.size {
                                // Calculate opacity based on distance and hardness
                                let alpha = if dist < self.size * (1.0 - self.hardness) {
                                    self.opacity
                                } else {
                                    let t = (self.size - dist) / (self.size * self.hardness);
                                    t * self.opacity
                                };
                                
                                if alpha > 0.0 {
                                    // Get source pixel
                                    let src_pixel = image.get_pixel(sx as u32, sy as u32);
                                    // Get destination pixel
                                    let dst_pixel = image.get_pixel(px as u32, py as u32);
                                    
                                    // Blend pixels
                                    let mut rgba = dst_pixel.0;
                                    
                                    // Simple alpha compositing
                                    for i in 0..4 {  // Include alpha channel
                                        rgba[i] = ((1.0 - alpha) * rgba[i] as f64 + 
                                                   alpha * src_pixel.0[i] as f64) as u8;
                                    }
                                    
                                    image.put_pixel(px as u32, py as u32, image::Rgba(rgba));
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

impl super::Tool for CloneTool {
    fn tool_type(&self) -> super::ToolType {
        super::ToolType::Clone
    }

    fn cursor(&self) -> &'static str {
        if self.source_point.is_some() {
            "clone"
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
            self.last_point = None;
            // Note: We don't clear source_point when deactivating
            // as we might want to remember the clone source
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        if button != 1 || !self.active {
            return;
        }
        
        // If Alt key is pressed (would be handled through event flags in real implementation)
        // set the source point instead of cloning
        // For now, we'll assume first click sets source, second click starts cloning
        if self.source_point.is_none() {
            self.set_source(x, y);
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

    fn mouse_up(&mut self, _x: f64, _y: f64, _button: u32) {
        // We keep the last_point for cursor display
    }

    fn key_press(&mut self, key: &str) {
        match key {
            "Escape" => {
                // Clear the last point but not the source
                self.last_point = None;
                self.destination_point = None;
            },
            "Alt+Escape" => {
                // Clear both source and last point - would need key combo detection in real impl
                self.source_point = None;
                self.destination_point = None;
                self.last_point = None;
            },
            _ => {}
        }
    }

    fn draw_preview(&self, context: &cairo::Context, canvas: &crate::core::Canvas) {
        if !self.active {
            return;
        }
        
        context.save();
        
        // Draw source point if we have one
        if let Some(src) = &self.source_point {
            // Draw a crosshair at the source point
            context.set_source_rgba(0.1, 0.8, 0.1, 0.7);
            context.set_line_width(1.5);
            
            // Horizontal line of crosshair
            context.move_to(src.x - 6.0, src.y);
            context.line_to(src.x + 6.0, src.y);
            context.stroke();
            
            // Vertical line of crosshair
            context.move_to(src.x, src.y - 6.0);
            context.line_to(src.x, src.y + 6.0);
            context.stroke();
            
            // Draw a circle at the source point
            context.arc(src.x, src.y, 3.0, 0.0, 2.0 * std::f64::consts::PI);
            context.stroke();
            
            // If we have both source and current point, draw a connecting line
            if let Some(point) = &self.last_point {
                // Draw a dashed line between source and destination
                context.set_source_rgba(0.1, 0.8, 0.1, 0.4);
                context.set_line_width(1.0);
                context.set_dash(&[5.0, 5.0], 0.0);
                context.move_to(src.x, src.y);
                context.line_to(point.x, point.y);
                context.stroke();
                
                // Reset dash pattern
                context.set_dash(&[], 0.0);
            }
        }
        
        // Draw cursor at current position
        if let Some(point) = &self.last_point {
            // Draw outer circle showing clone brush size
            context.set_source_rgba(0.2, 0.5, 0.9, 0.5);
            context.set_line_width(1.0);
            context.arc(point.x, point.y, self.size, 0.0, 2.0 * std::f64::consts::PI);
            context.stroke();
            
            // Draw inner circle showing hardness falloff
            let inner_radius = self.size * (1.0 - self.hardness);
            if inner_radius > 0.0 {
                context.set_source_rgba(0.2, 0.5, 0.9, 0.2);
                context.arc(point.x, point.y, inner_radius, 0.0, 2.0 * std::f64::consts::PI);
                context.stroke();
            }
        }
        
        context.restore();
    }
} 