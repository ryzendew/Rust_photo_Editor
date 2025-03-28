use uuid::Uuid;
use cairo::{Context, FontSlant, FontWeight as CairoFontWeight, TextExtents};
use crate::vector::{Point, Rect, VectorObject, Transform, SelectionState, Color};
use crate::vector::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextVerticalAlignment {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextWrap {
    None,
    Word,
    Character,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
}

impl From<FontWeight> for CairoFontWeight {
    fn from(weight: FontWeight) -> Self {
        match weight {
            FontWeight::Normal => CairoFontWeight::Normal,
            FontWeight::Bold => CairoFontWeight::Bold,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl From<FontStyle> for FontSlant {
    fn from(style: FontStyle) -> Self {
        match style {
            FontStyle::Normal => FontSlant::Normal,
            FontStyle::Italic => FontSlant::Italic,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: Color,
    pub alignment: TextAlignment,
    pub vertical_alignment: TextVerticalAlignment,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub paragraph_spacing: f64,
    pub wrapping: TextWrap,
    pub max_width: Option<f64>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Sans".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            color: Color::black(),
            alignment: TextAlignment::Left,
            vertical_alignment: TextVerticalAlignment::Top,
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            paragraph_spacing: 0.0,
            wrapping: TextWrap::Word,
            max_width: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextShape {
    pub id: String,
    pub text: String,
    pub position: Point,
    pub transform: Transform,
    pub style: TextStyle,
    pub selection_state: SelectionState,
    pub visible: bool,
    pub locked: bool,
    pub bounds: Option<Rect>,
    pub path: Path, // Path representation for the text outline
}

impl Default for TextShape {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text: "Text".to_string(),
            position: Point::new(0.0, 0.0),
            transform: Transform::identity(),
            style: TextStyle::default(),
            selection_state: SelectionState::None,
            visible: true,
            locked: false,
            bounds: None,
            path: Path::new(),
        }
    }
}

impl TextShape {
    pub fn new(text: String, position: Point, style: TextStyle) -> Self {
        let mut text_shape = Self {
            id: Uuid::new_v4().to_string(),
            text,
            position,
            style,
            ..Default::default()
        };
        
        // Create path representation
        text_shape.update_path();
        
        text_shape
    }
    
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.update_path();
    }
    
    pub fn set_style(&mut self, style: TextStyle) {
        self.style = style;
        self.update_path();
    }
    
    pub fn set_font_family(&mut self, font_family: String) {
        self.style.font_family = font_family;
        self.update_path();
    }
    
    pub fn set_font_size(&mut self, font_size: f64) {
        self.style.font_size = font_size;
        self.update_path();
    }
    
    pub fn set_color(&mut self, color: Color) {
        self.style.color = color;
    }
    
    pub fn set_alignment(&mut self, alignment: TextAlignment) {
        self.style.alignment = alignment;
        self.update_path();
    }
    
    pub fn calculate_bounds(&self, context: &Context) -> Rect {
        context.save();
        
        // Set up font
        context.select_font_face(
            &self.style.font_family, 
            self.style.font_style.into(), 
            self.style.font_weight.into()
        );
        context.set_font_size(self.style.font_size);
        
        // Get text extents
        let extents = context.text_extents(&self.text).unwrap();
        
        // Calculate bounds based on alignment and position
        let (x, y) = match self.style.alignment {
            TextAlignment::Left => (self.position.x, self.position.y),
            TextAlignment::Center => (self.position.x - extents.width() / 2.0, self.position.y),
            TextAlignment::Right => (self.position.x - extents.width(), self.position.y),
            TextAlignment::Justified => (self.position.x, self.position.y), // Same as left for single line
        };
        
        context.restore();
        
        // Create bounds rectangle
        Rect::new(
            x,
            y - extents.height(),
            extents.width(),
            extents.height()
        )
    }
    
    pub fn update_path(&mut self) {
        // Create a new in-memory surface to generate the path
        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            1, 1 // Dummy size
        ).unwrap();
        let context = Context::new(&surface).unwrap();
        
        // Calculate bounds
        let bounds = self.calculate_bounds(&context);
        self.bounds = Some(bounds);
        
        // For now, we just use a rectangular path as a placeholder
        // A full implementation would convert text to outlines
        let mut path = Path::new();
        path.add_point(bounds.x, bounds.y, crate::vector::path::PathNodeType::Point);
        path.add_point(bounds.x + bounds.width, bounds.y, crate::vector::path::PathNodeType::Point);
        path.add_point(bounds.x + bounds.width, bounds.y + bounds.height, crate::vector::path::PathNodeType::Point);
        path.add_point(bounds.x, bounds.y + bounds.height, crate::vector::path::PathNodeType::Point);
        path.set_closed(true);
        
        self.path = path;
    }
    
    pub fn draw(&self, context: &Context) {
        if !self.visible {
            return;
        }
        
        context.save();
        
        // Apply transforms
        context.translate(self.position.x, self.position.y);
        
        // Create a matrix using the transform fields
        let matrix = cairo::Matrix::new(
            self.transform.a,
            self.transform.b,
            self.transform.c,
            self.transform.d,
            self.transform.e,
            self.transform.f
        );
        context.transform(matrix);
        
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
        let (x, y) = match self.style.alignment {
            TextAlignment::Left => (0.0, 0.0),
            TextAlignment::Center => (-extents.width() / 2.0, 0.0),
            TextAlignment::Right => (-extents.width(), 0.0),
            TextAlignment::Justified => (0.0, 0.0), // Same as left for single line
        };
        
        // Draw text
        context.move_to(x, y);
        context.show_text(&self.text).unwrap();
        
        // Draw selection indicators if selected
        if self.selection_state != SelectionState::None {
            if let Some(bounds) = self.bounds {
                // Draw selection rectangle
                context.set_source_rgba(0.0, 0.7, 1.0, 0.5);
                context.set_line_width(1.0);
                context.rectangle(
                    -self.position.x + bounds.x - 2.0,
                    -self.position.y + bounds.y - 2.0, 
                    bounds.width + 4.0, 
                    bounds.height + 4.0
                );
                context.stroke();
                
                // Draw control points at corners
                if self.selection_state == SelectionState::EditPoints {
                    let corners = [
                        Point::new(bounds.x, bounds.y),
                        Point::new(bounds.x + bounds.width, bounds.y),
                        Point::new(bounds.x + bounds.width, bounds.y + bounds.height),
                        Point::new(bounds.x, bounds.y + bounds.height),
                    ];
                    
                    context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    for corner in &corners {
                        context.rectangle(
                            -self.position.x + corner.x - 3.0,
                            -self.position.y + corner.y - 3.0,
                            6.0,
                            6.0
                        );
                        context.fill();
                        
                        context.set_source_rgba(0.0, 0.7, 1.0, 1.0);
                        context.rectangle(
                            -self.position.x + corner.x - 3.0,
                            -self.position.y + corner.y - 3.0,
                            6.0,
                            6.0
                        );
                        context.stroke();
                    }
                }
            }
        }
        
        context.restore();
    }
}

impl VectorObject for TextShape {
    fn get_bounds(&self) -> Rect {
        self.bounds.unwrap_or_else(|| {
            // Create a temporary surface to calculate bounds
            let surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                1, 1 // Dummy size
            ).unwrap();
            let context = Context::new(&surface).unwrap();
            self.calculate_bounds(&context)
        })
    }
    
    fn contains_point(&self, point: &Point) -> bool {
        let bounds = self.get_bounds();
        bounds.contains(point)
    }
    
    fn transform(&mut self, transform: &Transform) {
        self.transform = transform.multiply(&self.transform);
        
        // Apply translation part to position
        let new_position = transform.apply_to_point(&self.position);
        self.position = new_position;
        
        self.update_path();
    }
    
    fn draw(&self, context: &Context) {
        self.draw(context);
    }
    
    fn clone_box(&self) -> Box<dyn VectorObject> {
        Box::new(self.clone())
    }
} 