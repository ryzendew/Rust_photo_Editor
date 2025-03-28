use crate::core::canvas::Tool;
use super::ToolImpl;

#[derive(Clone)]
pub struct TextTool {
    name: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    active: bool,
    position_x: f64,
    position_y: f64,
    text: String,
    font_family: String,
    font_size: f64,
    font_weight: String,
    text_color: (f64, f64, f64, f64), // RGBA
    is_editing: bool,
}

impl TextTool {
    pub fn new() -> Self {
        Self {
            name: "Text",
            icon: "insert-text-symbolic",
            tooltip: "Add vector text",
            active: false,
            position_x: 0.0,
            position_y: 0.0,
            text: "Text".to_string(),
            font_family: "Sans".to_string(),
            font_size: 24.0,
            font_weight: "normal".to_string(),
            text_color: (0.0, 0.0, 0.0, 1.0), // Black
            is_editing: false,
        }
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    
    pub fn set_font_family(&mut self, family: &str) {
        self.font_family = family.to_string();
    }
    
    pub fn set_font_size(&mut self, size: f64) {
        self.font_size = size;
    }
    
    pub fn set_font_weight(&mut self, weight: &str) {
        self.font_weight = weight.to_string();
    }
    
    pub fn set_text_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.text_color = (r, g, b, a);
    }
}

impl ToolImpl for TextTool {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn icon(&self) -> &'static str {
        self.icon
    }
    
    fn tooltip(&self) -> &'static str {
        self.tooltip
    }
    
    fn tool_type(&self) -> Tool {
        Tool::Text
    }
    
    fn activate(&mut self) {
        self.active = true;
    }
    
    fn deactivate(&mut self) {
        self.active = false;
        self.is_editing = false;
    }
    
    fn handle_mouse_down(&mut self, x: f64, y: f64, _button: u32) {
        self.position_x = x;
        self.position_y = y;
        self.is_editing = true;
        
        // In a real implementation, this would open a text editor or
        // place the cursor at the clicked position
    }
    
    fn handle_mouse_move(&mut self, _x: f64, _y: f64) {
        // Text tool typically doesn't do anything on mouse move
    }
    
    fn handle_mouse_up(&mut self, _x: f64, _y: f64, _button: u32) {
        // Text tool typically doesn't do anything on mouse up
        // unless it's clicking to select text
    }
    
    fn handle_key_press(&mut self, key: &str) {
        if self.is_editing {
            match key {
                "Escape" => {
                    self.is_editing = false;
                    // Cancel text editing
                },
                "Enter" => {
                    // Create a new line or commit the text
                },
                "Backspace" => {
                    // Delete the last character
                    if !self.text.is_empty() {
                        self.text.pop();
                    }
                },
                _ => {
                    if key.len() == 1 {
                        // Add the character to the text
                        self.text.push_str(key);
                    }
                }
            }
        }
    }
    
    fn handle_key_release(&mut self, _key: &str) {
        // No special handling needed for key release
    }
    
    fn on_mouse_down(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        if let Some(vector_doc) = &mut canvas.vector_document {
            let active_layer = vector_doc.get_active_layer();
            if let Some(layer) = active_layer {
                // Create a text shape at the clicked position
                let shape = VectorShape::rectangle(x, y, 100.0, 20.0);
                layer.add_shape(shape);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    fn on_mouse_drag(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        // Text tool doesn't need drag behavior
        false
    }
    
    fn on_mouse_up(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool {
        // Text tool doesn't need mouse up behavior
        false
    }
}

impl super::Tool for TextTool {
    fn tool_type(&self) -> super::ToolType {
        super::ToolType::Text
    }

    fn cursor(&self) -> &'static str {
        "text"
    }

    fn active(&self) -> bool {
        false // This should be replaced with an actual active field
    }

    fn set_active(&mut self, _active: bool) {
        // Set active state
    }

    fn mouse_down(&mut self, _x: f64, _y: f64, _button: u32) {
        // Mouse down handler
    }

    fn mouse_move(&mut self, _x: f64, _y: f64) {
        // Mouse move handler
    }

    fn mouse_up(&mut self, _x: f64, _y: f64, _button: u32) {
        // Mouse up handler
    }

    fn key_press(&mut self, _key: &str) {
        // Key press handler
    }

    fn draw_preview(&self, context: &cairo::Context, canvas: &crate::core::Canvas) {
        // Draw text preview
        context.save().unwrap();
        context.set_source_rgba(0.0, 0.0, 0.0, 0.8);
        context.new_path();
        context.restore().unwrap();
    }
} 