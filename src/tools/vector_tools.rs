use cairo::Context;
use gtk4::gdk::RGBA;
use crate::vector::{
    Point, Rect, Transform, SelectionState,
    VectorShape, ShapeType, FillStyle, StrokeStyle, Color, LineDash,
    TextShape, TextStyle, TextAlignment, FontWeight, FontStyle,
    PathNode, PathNodeType, BezierPoint,
    VectorDocument, VectorPath
};
use crate::vector::path::Path;
use crate::vector::shape::VectorShape as ShapeImpl;
use crate::core::Canvas;

/// Tool for creating rectangles
#[derive(Clone)]
pub struct RectangleTool {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub corner_radius: f64,
    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_width: f64,
}

impl Default for RectangleTool {
    fn default() -> Self {
        Self {
            start_point: None,
            end_point: None,
            corner_radius: 0.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 0.5),
            stroke_color: Color::black(),
            stroke_width: 1.0,
        }
    }
}

impl RectangleTool {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start(&mut self, x: f64, y: f64) {
        self.start_point = Some(Point::new(x, y));
        self.end_point = Some(Point::new(x, y));
    }
    
    pub fn update(&mut self, x: f64, y: f64) {
        self.end_point = Some(Point::new(x, y));
    }
    
    pub fn end(&mut self, document: &mut VectorDocument) -> bool {
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            if start.distance(&end) < 5.0 {
                // Too small to create a shape
                self.reset();
                return false;
            }
            
            // Calculate rectangle dimensions
            let rect = Rect::from_points(start, end);
            
            // Create the shape using the proper constructor
            let shape_impl = ShapeImpl::new_rectangle(
                rect.x, rect.y, rect.width, rect.height, self.corner_radius
            );
            
            // Convert to VectorShape enum that VectorDocument expects
            let shape = VectorShape::rectangle(rect.x, rect.y, rect.width, rect.height);
            
            // Add to document
            document.add_shape(shape);
            
            self.reset();
            return true;
        }
        
        false
    }
    
    pub fn cancel(&mut self) {
        self.reset();
    }
    
    pub fn reset(&mut self) {
        self.start_point = None;
        self.end_point = None;
    }
    
    pub fn draw_preview(&self, context: &Context) {
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            let rect = Rect::from_points(start, end);
            
            // Draw preview rectangle
            context.save();
            
            // Fill
            context.set_source_rgba(
                self.fill_color.r,
                self.fill_color.g,
                self.fill_color.b,
                self.fill_color.a
            );
            
            if self.corner_radius > 0.0 {
                let radius = self.corner_radius.min(rect.width / 2.0).min(rect.height / 2.0);
                
                context.new_sub_path();
                context.arc(rect.x + rect.width - radius, rect.y + radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
                context.arc(rect.x + rect.width - radius, rect.y + rect.height - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
                context.arc(rect.x + radius, rect.y + rect.height - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
                context.arc(rect.x + radius, rect.y + radius, radius, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
                context.close_path();
            } else {
                context.rectangle(rect.x, rect.y, rect.width, rect.height);
            }
            
            context.fill_preserve();
            
            // Stroke
            context.set_source_rgba(
                self.stroke_color.r,
                self.stroke_color.g,
                self.stroke_color.b,
                self.stroke_color.a
            );
            context.set_line_width(self.stroke_width);
            context.stroke();
            
            context.restore();
        }
    }
}

/// Tool for creating ellipses
#[derive(Clone)]
pub struct EllipseTool {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_width: f64,
}

impl Default for EllipseTool {
    fn default() -> Self {
        Self {
            start_point: None,
            end_point: None,
            fill_color: Color::new(1.0, 1.0, 1.0, 0.5),
            stroke_color: Color::black(),
            stroke_width: 1.0,
        }
    }
}

impl EllipseTool {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start(&mut self, x: f64, y: f64) {
        self.start_point = Some(Point::new(x, y));
        self.end_point = Some(Point::new(x, y));
    }
    
    pub fn update(&mut self, x: f64, y: f64) {
        self.end_point = Some(Point::new(x, y));
    }
    
    pub fn end(&mut self, document: &mut VectorDocument) -> bool {
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            if start.distance(&end) < 5.0 {
                // Too small to create a shape
                self.reset();
                return false;
            }
            
            // Calculate ellipse dimensions
            let rect = Rect::from_points(start, end);
            let radius_x = rect.width / 2.0;
            let radius_y = rect.height / 2.0;
            let center = rect.center();
            
            // Create the shape using VectorShape enum directly
            let shape = VectorShape::ellipse(center.x, center.y, radius_x, radius_y);
            
            // Add to document
            document.add_shape(shape);
            
            self.reset();
            return true;
        }
        
        false
    }
    
    pub fn cancel(&mut self) {
        self.reset();
    }
    
    pub fn reset(&mut self) {
        self.start_point = None;
        self.end_point = None;
    }
    
    pub fn draw_preview(&self, context: &Context) {
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            let rect = Rect::from_points(start, end);
            let center = rect.center();
            let radius_x = rect.width / 2.0;
            let radius_y = rect.height / 2.0;
            
            // Draw preview ellipse
            context.save();
            
            // Transform to make a proper ellipse
            context.translate(center.x, center.y);
            context.scale(radius_x, radius_y);
            
            // Draw
            context.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
            
            // Fill
            context.set_source_rgba(
                self.fill_color.r,
                self.fill_color.g,
                self.fill_color.b,
                self.fill_color.a
            );
            context.fill_preserve();
            
            // Stroke
            context.set_source_rgba(
                self.stroke_color.r,
                self.stroke_color.g,
                self.stroke_color.b,
                self.stroke_color.a
            );
            context.set_line_width(self.stroke_width / radius_x.min(radius_y)); // Adjust for scale
            context.stroke();
            
            context.restore();
        }
    }
}

/// Tool for creating text objects
#[derive(Clone)]
pub struct TextTool {
    pub position: Option<Point>,
    pub text: String,
    pub style: TextStyle,
}

impl Default for TextTool {
    fn default() -> Self {
        Self {
            position: None,
            text: "Text".to_string(),
            style: TextStyle::default(),
        }
    }
}

impl TextTool {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn click(&mut self, x: f64, y: f64, document: &mut VectorDocument) -> bool {
        self.position = Some(Point::new(x, y));
        
        // Get active layer
        if let Some(layer) = document.get_active_layer_mut() {
            // Create text shape using VectorShape's methods
            // Since VectorShape enum doesn't seem to have a specific text constructor,
            // we need to use a different approach or potentially extend the enum
            
            // For now, we'll use a rectangle as a placeholder
            // In a real implementation, you might want to add a Text variant to the VectorShape enum
            let shape = VectorShape::rectangle(x, y, 100.0, 20.0);
            
            // Add shape to layer
            layer.add_shape(shape);
            
            self.reset();
            return true;
        }
        
        false
    }
    
    pub fn reset(&mut self) {
        self.position = None;
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    
    pub fn set_font_family(&mut self, family: &str) {
        self.style.font_family = family.to_string();
    }
    
    pub fn set_font_size(&mut self, size: f64) {
        self.style.font_size = size;
    }
    
    pub fn set_font_weight(&mut self, weight: FontWeight) {
        self.style.font_weight = weight;
    }
    
    pub fn set_color(&mut self, color: Color) {
        self.style.color = color;
    }
    
    pub fn set_alignment(&mut self, alignment: TextAlignment) {
        self.style.alignment = alignment;
    }
    
    pub fn draw_preview(&self, context: &Context, x: f64, y: f64) {
        // Draw preview of text at cursor position
        context.save();
        
        // Set up font
        context.select_font_face(
            &self.style.font_family, 
            self.style.font_style.into(), 
            self.style.font_weight.into()
        );
        context.set_font_size(self.style.font_size);
        
        // Set color
        context.set_source_rgba(
            self.style.color.r,
            self.style.color.g,
            self.style.color.b,
            self.style.color.a
        );
        
        // Get text extents
        let extents = context.text_extents(&self.text).unwrap();
        
        // Position text based on alignment
        let (pos_x, pos_y) = match self.style.alignment {
            TextAlignment::Left => (x, y),
            TextAlignment::Center => (x - extents.width() / 2.0, y),
            TextAlignment::Right => (x - extents.width(), y),
            TextAlignment::Justified => (x, y), // Same as left for single line
        };
        
        // Draw text
        context.move_to(pos_x, pos_y);
        context.show_text(&self.text).unwrap();
        
        // Draw cursor indicator
        context.set_source_rgba(0.0, 0.7, 1.0, 0.7);
        context.rectangle(x - 5.0, y - 5.0, 10.0, 10.0);
        context.stroke();
        
        context.restore();
    }
}

/// Tool for creating paths
#[derive(Clone)]
pub struct PathTool {
    pub path: VectorPath,
    pub current_point: Option<Point>,
    pub is_drawing: bool,
    pub node_type: PathNodeType,
    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_width: f64,
}

impl Default for PathTool {
    fn default() -> Self {
        Self {
            path: VectorPath::new(),
            current_point: None,
            is_drawing: false,
            node_type: PathNodeType::Point,
            fill_color: Color::new(0.0, 0.0, 0.0, 1.0),
            stroke_color: Color::new(0.0, 0.0, 0.0, 1.0),
            stroke_width: 1.0,
        }
    }
}

impl PathTool {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start(&mut self, x: f64, y: f64) {
        if !self.is_drawing {
            // Start a new path
            self.path = VectorPath::new();
            self.is_drawing = true;
        }
        
        // Add point to path
        self.path.add_point(x, y, self.node_type);
        self.current_point = Some(Point::new(x, y));
    }
    
    pub fn add_point(&mut self, x: f64, y: f64) {
        if self.is_drawing {
            self.path.add_point(x, y, self.node_type);
            self.current_point = Some(Point::new(x, y));
        }
    }
    
    pub fn end(&mut self, closed: bool, document: &mut VectorDocument) -> bool {
        if self.path.is_empty() {
            return false;
        }
        
        self.path.set_closed(closed);
        
        // Create shape using VectorShape enum
        let shape = VectorShape::Path {
            path: self.path.clone()
        };
        
        // Set fill if closed
        // Note: we're not using set_fill here since the shape is the enum version
        
        // Add to document
        document.add_shape(shape);
        
        self.reset();
        true
    }
    
    pub fn cancel(&mut self) {
        self.reset();
    }
    
    pub fn reset(&mut self) {
        self.path = VectorPath::new();
        self.current_point = None;
        self.is_drawing = false;
    }
    
    pub fn set_node_type(&mut self, node_type: PathNodeType) {
        self.node_type = node_type;
    }
    
    pub fn draw_preview(&self, context: &Context, current_x: f64, current_y: f64) {
        if self.is_drawing {
            context.save();
            
            // Draw the path so far
            self.path.build_path(context);
            
            // If we have a current point, draw line to current mouse position
            if let Some(last) = self.current_point {
                context.move_to(last.x, last.y);
                context.line_to(current_x, current_y);
            }
            
            // Stroke the preview
            context.set_source_rgba(
                self.stroke_color.r,
                self.stroke_color.g,
                self.stroke_color.b,
                self.stroke_color.a
            );
            context.set_line_width(self.stroke_width);
            context.stroke();
            
            // Draw the nodes
            self.path.draw_nodes(context);
            
            // Draw the current position
            context.set_source_rgba(0.0, 0.7, 1.0, 1.0);
            context.arc(current_x, current_y, 3.0, 0.0, 2.0 * std::f64::consts::PI);
            context.fill();
            
            context.restore();
        }
    }
} 