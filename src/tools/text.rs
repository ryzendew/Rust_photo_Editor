use crate::core::Canvas;
use crate::vector::{Point, TextShape, TextStyle, FontWeight, VectorShape};
use crate::tools::{Tool, ToolType};
use super::ToolImpl;
use cairo;

#[derive(Clone)]
pub struct TextTool {
    position: Option<Point>,
    text: Option<String>,
    font_family: String,
    font_size: f64,
    font_weight: FontWeight,
    text_color: (f64, f64, f64, f64),
    is_editing: bool,
}

impl Default for TextTool {
    fn default() -> Self {
        Self {
            position: None,
            text: None,
            font_family: "Sans".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            text_color: (0.0, 0.0, 0.0, 1.0),
            is_editing: false,
        }
    }
}

impl TextTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    pub fn set_font_family(&mut self, family: String) {
        self.font_family = family;
    }

    pub fn set_font_size(&mut self, size: f64) {
        self.font_size = size;
    }

    pub fn set_font_weight(&mut self, weight: FontWeight) {
        self.font_weight = weight;
    }

    pub fn set_text_color(&mut self, color: (f64, f64, f64, f64)) {
        self.text_color = color;
    }

    fn draw_preview(&self, context: &cairo::Context) {
        if let (Some(position), Some(text)) = (&self.position, &self.text) {
            context.save();

            // Set font properties
            context.select_font_face(&self.font_family, cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            context.set_font_size(self.font_size);
            context.set_source_rgba(self.text_color.0, self.text_color.1, self.text_color.2, self.text_color.3);

            // Get text extents for cursor positioning
            let extents = context.text_extents(text).expect("Failed to get text extents");
            let cursor_x = position.x + extents.width();

            // Draw text
            context.move_to(position.x, position.y);
            context.show_text(text).expect("Failed to draw text");

            // Draw cursor
            context.set_line_width(1.0);
            context.move_to(cursor_x, position.y - extents.height() * 0.2);
            context.line_to(cursor_x, position.y + extents.height() * 0.2);
            context.stroke().expect("Failed to draw cursor");

            context.restore();
        }
    }
}

impl Tool for TextTool {
    fn tool_type(&self) -> ToolType {
        ToolType::Text
    }

    fn cursor(&self) -> &'static str {
        "text"
    }

    fn active(&self) -> bool {
        self.is_editing
    }

    fn set_active(&mut self, active: bool) {
        self.is_editing = active;
        if !active {
            self.position = None;
            self.text = None;
        }
    }

    fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        if button != 1 || !self.is_editing {
            return;
        }
        self.position = Some(Point::new(x, y));
    }

    fn mouse_move(&mut self, _x: f64, _y: f64) {
        // No action needed for text tool mouse move
    }

    fn mouse_up(&mut self, _x: f64, _y: f64, _button: u32) {
        // No action needed for text tool mouse up
    }

    fn key_press(&mut self, key: &str) {
        if !self.is_editing {
            return;
        }

        match key {
            "Escape" => {
                self.is_editing = false;
                self.position = None;
                self.text = None;
            }
            "Return" => {
                self.is_editing = false;
            }
            "BackSpace" => {
                if let Some(text) = &mut self.text {
                    text.pop();
                }
            }
            _ if key.len() == 1 => {
                let text = self.text.get_or_insert_with(String::new);
                text.push_str(key);
            }
            _ => {}
        }
    }

    fn draw_preview(&self, context: &cairo::Context, _canvas: &Canvas) {
        self.draw_preview(context);
    }
}

impl ToolImpl for TextTool {
    fn on_mouse_down(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        self.position = Some(Point::new(x, y));
        self.is_editing = true;
        true
    }
    
    fn on_mouse_drag(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        false
    }
    
    fn on_mouse_up(&mut self, canvas: &mut Canvas, _x: f64, _y: f64) -> bool {
        if !self.is_editing {
            return false;
        }

        if let (Some(text), Some(pos)) = (&self.text, &self.position) {
            if text.is_empty() {
                return false;
            }

            if let Some(doc) = &mut canvas.vector_document {
                let text_shape = VectorShape::Text {
                    text: text.clone(),
                    x: pos.x,
                    y: pos.y,
                    font_family: self.font_family.clone(),
                    font_size: self.font_size,
                    font_weight: self.font_weight.to_string(),
                };

                if let Some(layer) = doc.get_active_layer_mut() {
                    layer.add_shape(text_shape);
                    self.is_editing = false;
                    self.text = None;
                    self.position = None;
                    return true;
                }
            }
        }
        false
    }
    
    fn get_cursor(&self) -> Option<String> {
        Some("text".to_string())
    }
    
    fn draw(&self, canvas: &Canvas, context: &cairo::Context) {
        if self.is_editing {
            context.save().expect("Failed to save context state");
            
            // Set text properties
            context.set_font_size(self.font_size);
            context.select_font_face(&self.font_family, cairo::FontSlant::Normal, self.font_weight.into());
            
            let (r, g, b, a) = self.text_color;
            context.set_source_rgba(r, g, b, a);
            
            if let Some(pos) = &self.position {
                // Draw the text
                if let Some(text) = &self.text {
                    context.move_to(pos.x, pos.y);
                    context.show_text(text).expect("Failed to show text");
                    
                    // Draw cursor
                    let extents = context.text_extents(text).expect("Failed to get text extents");
                    let cursor_x = pos.x + extents.width();
                    context.move_to(cursor_x, pos.y - extents.height());
                    context.line_to(cursor_x, pos.y);
                    context.stroke().expect("Failed to stroke cursor");
                }
            }
            
            context.restore().expect("Failed to restore context state");
        }
    }
}

// Add a helper method to convert FontWeight to cairo::FontWeight
impl FontWeight {
    fn to_cairo_font_weight(&self) -> cairo::FontWeight {
        match self {
            FontWeight::Normal => cairo::FontWeight::Normal,
            FontWeight::Bold => cairo::FontWeight::Bold,
        }
    }

    fn into(self) -> cairo::FontWeight {
        self.to_cairo_font_weight()
    }
} 