use crate::core::Canvas;
use crate::vector::Point;
use super::ToolImpl;

#[derive(Clone)]
pub struct BrushTool {
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
    pub color: [u8; 4],
    pub last_point: Option<Point>,
    pub active: bool,
}

impl BrushTool {
    pub fn new() -> Self {
        Self {
            size: 10.0,
            hardness: 0.5,
            opacity: 1.0,
            color: [0, 0, 0, 255],
            last_point: None,
            active: false,
        }
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.last_point = None;
        }
    }
}

impl ToolImpl for BrushTool {
    fn on_mouse_down(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        let point = Point::new(x, y);
        self.last_point = Some(point);
        
        // Draw a single dot at the current position
        if let Some(layer) = canvas.layer_manager.get_active_layer_mut() {
            // Draw a circle at the current position
            // In a real implementation, we'd draw an anti-aliased circle using the brush parameters
            // For now, just place a simple marker
            let buffer = &mut layer.image;
            let size = self.size as i32;
            let cx = x as i32;
            let cy = y as i32;
            
            // Simple circle drawing
            for dy in -size..=size {
                for dx in -size..=size {
                    let px = cx + dx;
                    let py = cy + dy;
                    
                    // Check if in bounds
                    if px >= 0 && px < buffer.width() as i32 && 
                       py >= 0 && py < buffer.height() as i32 {
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
                                let pixel = buffer.get_pixel(px as u32, py as u32);
                                let mut rgba = pixel.0;
                                
                                // Simple alpha compositing
                                for i in 0..3 {
                                    rgba[i] = ((1.0 - alpha) * rgba[i] as f64 + 
                                              alpha * self.color[i] as f64) as u8;
                                }
                                
                                buffer.put_pixel(px as u32, py as u32, image::Rgba(rgba));
                            }
                        }
                    }
                }
            }
        }
        
        true
    }
    
    fn on_mouse_drag(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if let Some(last) = self.last_point {
            // Draw a line from last point to current point
            // In a real implementation, we'd use a line drawing algorithm with proper anti-aliasing
            // For simplicity, we'll just call on_mouse_down repeatedly along the line
            
            let curr = Point::new(x, y);
            let dist = last.distance_to(&curr);
            let step_size = self.size / 4.0;  // Make steps smaller than brush size for smooth lines
            
            if dist > 0.0 {
                let steps = (dist / step_size).ceil() as usize;
                
                for i in 0..=steps {
                    let t = if steps == 0 { 0.0 } else { i as f64 / steps as f64 };
                    let ix = last.x + (curr.x - last.x) * t;
                    let iy = last.y + (curr.y - last.y) * t;
                    
                    self.on_mouse_down(canvas, ix, iy);
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
        Some("brush".to_string())
    }
}

impl super::Tool for BrushTool {
    fn tool_type(&self) -> super::ToolType {
        super::ToolType::Brush
    }

    fn cursor(&self) -> &'static str {
        "brush"
    }

    fn active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
        
        if !active {
            self.last_point = None;
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        if button != 1 || !self.active {
            return;
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
        if key == "Escape" {
            self.last_point = None;
        }
    }

    fn draw_preview(&self, context: &cairo::Context, canvas: &crate::core::Canvas) {
        if !self.active {
            return;
        }
        
        if let Some(point) = &self.last_point {
            context.save();
            
            // Draw outer circle showing brush size
            context.set_source_rgba(
                self.color[0] as f64 / 255.0,
                self.color[1] as f64 / 255.0,
                self.color[2] as f64 / 255.0,
                0.5
            );
            context.set_line_width(1.0);
            context.arc(point.x, point.y, self.size, 0.0, 2.0 * std::f64::consts::PI);
            context.stroke();
            
            // Draw inner circle showing hardness falloff
            let inner_radius = self.size * (1.0 - self.hardness);
            if inner_radius > 0.0 {
                context.set_source_rgba(
                    self.color[0] as f64 / 255.0,
                    self.color[1] as f64 / 255.0,
                    self.color[2] as f64 / 255.0,
                    0.2
                );
                context.arc(point.x, point.y, inner_radius, 0.0, 2.0 * std::f64::consts::PI);
                context.stroke();
            }
            
            context.restore();
        }
    }
} 