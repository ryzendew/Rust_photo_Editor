use super::{VectorPath, VectorText, Fill};
use cairo::Context;
use std::f64::consts::PI;
use cairo::{LineCap, LineJoin, Pattern};
use uuid::Uuid;
use crate::vector::{Point, Rect, Transform, VectorObject, SelectionState, PathOperation};
use crate::vector::path::Path;

/// Represents a vector shape in the document.
/// Shapes can be rectangles, ellipses, paths, or text.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeType {
    Rectangle { width: f64, height: f64, corner_radius: f64 },
    Ellipse { radius_x: f64, radius_y: f64 },
    Circle { radius: f64 },
    Polygon { sides: usize, radius: f64 },
    Star { outer_radius: f64, inner_radius: f64, points: usize },
    Custom { path: Path },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,  // 0.0 - 1.0
    pub g: f64,  // 0.0 - 1.0
    pub b: f64,  // 0.0 - 1.0
    pub a: f64,  // 0.0 - 1.0
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: 1.0,
        }
    }
    
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a as f64 / 255.0,
        }
    }
    
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
    
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
    
    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GradientType {
    Linear { start: Point, end: Point },
    Radial { center: Point, radius: f64 },
    Conical { center: Point, angle: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub gradient_type: GradientType,
    pub stops: Vec<(f64, Color)>, // position (0-1), color
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            gradient_type: GradientType::Linear { start: Point::new(0.0, 0.0), end: Point::new(1.0, 1.0) },
            stops: vec![(0.0, Color::black()), (1.0, Color::white())],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FillStyle {
    None,
    Solid(Color),
    Gradient(Gradient),
    Pattern(String), // Path to image or pattern name
}

impl Default for FillStyle {
    fn default() -> Self {
        FillStyle::Solid(Color::black())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineDash {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeStyle {
    pub width: f64,
    pub color: Color,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
    pub line_dash: LineDash,
    pub dash_pattern: Vec<f64>,
    pub dash_offset: f64,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color::black(),
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            line_dash: LineDash::Solid,
            dash_pattern: Vec::new(),
            dash_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorShape {
    pub id: String,
    pub name: String,
    pub shape_type: ShapeType,
    pub position: Point,
    pub transform: Transform,
    pub fill: FillStyle,
    pub stroke: StrokeStyle,
    pub selection_state: SelectionState,
    pub visible: bool,
    pub locked: bool,
    pub path_operation: PathOperation,
}

impl Default for VectorShape {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Shape".to_string(),
            shape_type: ShapeType::Rectangle { width: 100.0, height: 100.0, corner_radius: 0.0 },
            position: Point::new(0.0, 0.0),
            transform: Transform::identity(),
            fill: FillStyle::default(),
            stroke: StrokeStyle::default(),
            selection_state: SelectionState::None,
            visible: true,
            locked: false,
            path_operation: PathOperation::None,
        }
    }
}

impl VectorShape {
    /// Create a new rectangle shape
    pub fn new_rectangle(
        x: f64, 
        y: f64, 
        width: f64, 
        height: f64, 
        radius: f64
    ) -> Self {
        let mut shape = Self::default();
        shape.position = Point::new(x, y);
        shape.shape_type = ShapeType::Rectangle { 
            width, 
            height, 
            corner_radius: radius,
        };
        shape
    }
    
    /// Create a new ellipse shape
    pub fn new_ellipse(
        center_x: f64, 
        center_y: f64, 
        radius_x: f64, 
        radius_y: f64
    ) -> Self {
        let mut shape = Self::default();
        shape.position = Point::new(center_x, center_y);
        shape.shape_type = ShapeType::Ellipse { 
            radius_x, 
            radius_y,
        };
        shape
    }
    
    /// Create a new text shape
    pub fn new_text(
        x: f64,
        y: f64,
        text: &str,
        font_family: &str,
        font_size: f64,
        font_weight: &str
    ) -> Self {
        let text_obj = VectorText::new(text, font_family, font_size, font_weight, x, y);
        let mut shape = Self::default();
        shape.position = Point::new(x, y);
        shape.shape_type = ShapeType::Custom { path: text_obj.path };
        shape
    }
    
    /// Set fill properties for the shape
    pub fn set_fill(&mut self, fill: FillStyle) {
        self.fill = fill;
    }
    
    /// Set stroke properties for the shape
    pub fn set_stroke(&mut self, stroke: StrokeStyle) {
        self.stroke = stroke;
    }
    
    /// Set selection state for the shape
    pub fn set_selection_state(&mut self, state: SelectionState) {
        self.selection_state = state;
    }
    
    /// Draw the shape to the provided Cairo context
    pub fn draw(&self, cr: &Context) {
        if !self.visible {
            return;
        }
        
        cr.save().expect("Failed to save context");
        
        // Apply transforms
        cr.translate(self.position.x, self.position.y);
        
        // Create a matrix using the transform fields
        let matrix = cairo::Matrix::new(
            self.transform.a,
            self.transform.b,
            self.transform.c,
            self.transform.d,
            self.transform.e,
            self.transform.f
        );
        cr.transform(matrix);
        
        // Build the path
        self.build_path(cr);
        
        // Apply fill
        self.apply_fill(cr);
        
        // Apply stroke
        self.apply_stroke(cr);
        
        // Draw selection indicators if selected
        if self.selection_state != SelectionState::None {
            let bounds = self.get_bounds();
            
            // Draw selection rectangle
            cr.set_source_rgba(0.0, 0.7, 1.0, 0.5);
            cr.set_line_width(1.0);
            cr.rectangle(
                -self.position.x + bounds.x - 2.0,
                -self.position.y + bounds.y - 2.0, 
                bounds.width + 4.0, 
                bounds.height + 4.0
            );
            cr.stroke();
            
            // Draw control points at corners if in edit mode
            if self.selection_state == SelectionState::EditPoints {
                let corners = [
                    Point::new(bounds.x, bounds.y),
                    Point::new(bounds.x + bounds.width, bounds.y),
                    Point::new(bounds.x + bounds.width, bounds.y + bounds.height),
                    Point::new(bounds.x, bounds.y + bounds.height),
                ];
                
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                for corner in &corners {
                    cr.rectangle(
                        -self.position.x + corner.x - 3.0,
                        -self.position.y + corner.y - 3.0,
                        6.0,
                        6.0
                    );
                    cr.fill();
                    
                    cr.set_source_rgba(0.0, 0.7, 1.0, 1.0);
                    cr.rectangle(
                        -self.position.x + corner.x - 3.0,
                        -self.position.y + corner.y - 3.0,
                        6.0,
                        6.0
                    );
                    cr.stroke();
                }
            }
        }
        
        cr.restore().expect("Failed to restore context");
    }
    
    fn build_path(&self, context: &Context) {
        match &self.shape_type {
            ShapeType::Rectangle { width, height, corner_radius } => {
                if *corner_radius <= 0.0 {
                    // Simple rectangle
                    context.rectangle(0.0, 0.0, *width, *height);
                } else {
                    // Rounded rectangle
                    let radius = corner_radius.min(*width / 2.0).min(*height / 2.0);
                    
                    context.new_sub_path();
                    context.arc(*width - radius, radius, radius, -PI/2.0, 0.0);
                    context.arc(*width - radius, *height - radius, radius, 0.0, PI/2.0);
                    context.arc(radius, *height - radius, radius, PI/2.0, PI);
                    context.arc(radius, radius, radius, PI, 3.0*PI/2.0);
                    context.close_path();
                }
            },
            ShapeType::Ellipse { radius_x, radius_y } => {
                // Scale circle to create ellipse
                context.save();
                context.scale(*radius_x, *radius_y);
                context.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI);
                context.restore();
            },
            ShapeType::Circle { radius } => {
                context.arc(0.0, 0.0, *radius, 0.0, 2.0 * PI);
            },
            ShapeType::Polygon { sides, radius } => {
                if *sides < 3 {
                    return; // Invalid polygon
                }
                
                let angle_step = 2.0 * PI / *sides as f64;
                
                context.move_to(*radius, 0.0);
                
                for i in 1..*sides {
                    let angle = angle_step * i as f64;
                    let x = radius * angle.cos();
                    let y = radius * angle.sin();
                    context.line_to(x, y);
                }
                
                context.close_path();
            },
            ShapeType::Star { outer_radius, inner_radius, points } => {
                if *points < 3 {
                    return; // Invalid star
                }
                
                let angle_step = PI / *points as f64;
                
                context.move_to(*outer_radius, 0.0);
                
                for i in 1..(*points * 2) {
                    let angle = angle_step * i as f64;
                    let radius = if i % 2 == 1 { *inner_radius } else { *outer_radius };
                    let x = radius * angle.cos();
                    let y = radius * angle.sin();
                    context.line_to(x, y);
                }
                
                context.close_path();
            },
            ShapeType::Custom { path } => {
                path.build_path(context);
            },
        }
    }
    
    fn apply_fill(&self, context: &Context) {
        match &self.fill {
            FillStyle::None => {
                // No fill
                return;
            },
            FillStyle::Solid(color) => {
                context.set_source_rgba(
                    color.r,
                    color.g,
                    color.b,
                    color.a
                );
            },
            FillStyle::Gradient(gradient) => {
                // Set up gradient pattern
                match &gradient.gradient_type {
                    GradientType::Linear { start, end } => {
                        let linear = cairo::LinearGradient::new(
                            start.x, start.y,
                            end.x, end.y
                        );
                        
                        for &(offset, ref color) in &gradient.stops {
                            linear.add_color_stop_rgba(
                                offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        
                        context.set_source(&linear);
                    },
                    GradientType::Radial { center, radius } => {
                        let radial = cairo::RadialGradient::new(
                            center.x, center.y, 0.0,
                            center.x, center.y, 
                            *radius
                        );
                        
                        for &(offset, ref color) in &gradient.stops {
                            radial.add_color_stop_rgba(
                                offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        
                        context.set_source(&radial);
                    },
                    GradientType::Conical { center, angle: _ } => {
                        // Create a stable, owned Color value before using it
                        let default_color = Color::new(0.0, 0.0, 0.0, 1.0);
                        let color = gradient.stops.first().map(|(_, c)| c.clone()).unwrap_or(default_color);
                        context.set_source_rgba(
                            color.r,
                            color.g,
                            color.b,
                            color.a
                        );
                    }
                };
            },
            FillStyle::Pattern(_path) => {
                // For pattern fills, we would load an image or pattern
                // Just use a default fill for now
                context.set_source_rgba(0.8, 0.8, 0.8, 1.0);
            }
        }
        
        // Fill the path
        context.fill_preserve();
    }
    
    fn apply_stroke(&self, context: &Context) {
        // Set line properties
        context.set_line_width(self.stroke.width);
        context.set_line_cap(self.stroke.line_cap);
        context.set_line_join(self.stroke.line_join);
        context.set_miter_limit(self.stroke.miter_limit);
        
        // Set dash pattern if needed
        match self.stroke.line_dash {
            LineDash::Solid => {
                // Default: no dash
            },
            LineDash::Dashed => {
                context.set_dash(&[10.0, 5.0], 0.0);
            },
            LineDash::Dotted => {
                context.set_dash(&[2.0, 4.0], 0.0);
            },
            LineDash::DashDot => {
                context.set_dash(&[10.0, 5.0, 2.0, 5.0], 0.0);
            },
            LineDash::None => {
                // No stroke
                return;
            }
        }
        
        // If a custom dash pattern is provided, use it instead
        if !self.stroke.dash_pattern.is_empty() {
            context.set_dash(&self.stroke.dash_pattern, self.stroke.dash_offset);
        }
        
        // Apply the stroke
        context.stroke().expect("Failed to apply stroke");
    }

    pub fn draw_path(&self, context: &Context) {
        if let ShapeType::Custom { path } = &self.shape_type {
            if path.nodes.is_empty() {
                return;
            }
            
            // Move to the first point
            let first_node = &path.nodes[0];
            context.move_to(first_node.point.position.x, first_node.point.position.y);
            
            // Draw each segment
            for i in 1..path.nodes.len() {
                let prev_node = &path.nodes[i-1];
                let current_node = &path.nodes[i];
                
                // Check if control points differ from position (indicating a curve)
                let prev_has_control_out = 
                    prev_node.point.control_out.x != prev_node.point.position.x || 
                    prev_node.point.control_out.y != prev_node.point.position.y;
                
                let curr_has_control_in = 
                    current_node.point.control_in.x != current_node.point.position.x || 
                    current_node.point.control_in.y != current_node.point.position.y;
                
                if prev_has_control_out && curr_has_control_in {
                    // We have both control points, use a cubic Bezier curve
                    context.curve_to(
                        prev_node.point.control_out.x, 
                        prev_node.point.control_out.y,
                        current_node.point.control_in.x, 
                        current_node.point.control_in.y,
                        current_node.point.position.x, 
                        current_node.point.position.y
                    );
                } else if prev_has_control_out {
                    // Only have out control point, use quadratic Bezier approximation
                    context.curve_to(
                        prev_node.point.control_out.x,
                        prev_node.point.control_out.y,
                        prev_node.point.control_out.x,
                        prev_node.point.control_out.y,
                        current_node.point.position.x,
                        current_node.point.position.y
                    );
                } else if curr_has_control_in {
                    // Only have in control point, use quadratic Bezier approximation
                    context.curve_to(
                        current_node.point.control_in.x,
                        current_node.point.control_in.y,
                        current_node.point.control_in.x,
                        current_node.point.control_in.y,
                        current_node.point.position.x,
                        current_node.point.position.y
                    );
                } else {
                    // No control points, just use a line
                    context.line_to(
                        current_node.point.position.x,
                        current_node.point.position.y
                    );
                }
            }
            
            // Close the path if it's a closed shape
            if path.closed && path.nodes.len() > 2 {
                context.close_path();
            }
        }
    }

    fn update_control_points(&mut self, node_idx: usize, control_in: Point, control_out: Point) {
        if let ShapeType::Custom { path } = &mut self.shape_type {
            if let Some(node) = path.nodes.get_mut(node_idx) {
                node.point.control_in = control_in;
                node.point.control_out = control_out;
            }
        }
    }

    pub fn transform_path(&mut self, transform: &Transform) {
        if let ShapeType::Custom { path } = &mut self.shape_type {
            for node in &mut path.nodes {
                let matrix = transform.to_cairo_matrix();
                let (x, y) = matrix.transform_point(node.point.position.x, node.point.position.y);
                node.point.position.x = x;
                node.point.position.y = y;
                
                // Control points are direct Point values, not Options
                let (x, y) = matrix.transform_point(node.point.control_in.x, node.point.control_in.y);
                node.point.control_in.x = x;
                node.point.control_in.y = y;
                
                let (x, y) = matrix.transform_point(node.point.control_out.x, node.point.control_out.y);
                node.point.control_out.x = x;
                node.point.control_out.y = y;
            }
        }
    }
}

impl VectorObject for VectorShape {
    fn get_bounds(&self) -> Rect {
        match &self.shape_type {
            ShapeType::Rectangle { width, height, corner_radius } => {
                Rect::new(self.position.x, self.position.y, *width, *height)
            },
            ShapeType::Ellipse { radius_x, radius_y } => {
                Rect::new(
                    self.position.x - radius_x,
                    self.position.y - radius_y,
                    radius_x * 2.0,
                    radius_y * 2.0
                )
            },
            ShapeType::Circle { radius } => {
                Rect::new(
                    self.position.x - radius,
                    self.position.y - radius,
                    radius * 2.0,
                    radius * 2.0
                )
            },
            ShapeType::Polygon { radius, .. } => {
                Rect::new(
                    self.position.x - radius,
                    self.position.y - radius,
                    radius * 2.0,
                    radius * 2.0
                )
            },
            ShapeType::Star { outer_radius, .. } => {
                Rect::new(
                    self.position.x - outer_radius,
                    self.position.y - outer_radius,
                    outer_radius * 2.0,
                    outer_radius * 2.0
                )
            },
            ShapeType::Custom { path } => {
                // Get the path bounds and adjust for position
                let path_bounds = path.get_bounds();
                Rect::new(
                    self.position.x + path_bounds.x,
                    self.position.y + path_bounds.y,
                    path_bounds.width,
                    path_bounds.height
                )
            },
        }
    }
    
    fn contains_point(&self, point: &Point) -> bool {
        // Transform the point to shape's local coordinates
        let bounds = self.get_bounds();
        bounds.contains(point)
        
        // For more accurate hit testing, we would need to create a path and use cairo's in_fill method
    }
    
    fn transform(&mut self, transform: &Transform) {
        self.transform = transform.multiply(&self.transform);
        
        // Apply translation part to position
        let new_position = transform.apply_to_point(&self.position);
        self.position = new_position;
    }
    
    fn clone_box(&self) -> Box<dyn VectorObject> {
        Box::new(self.clone())
    }
    
    fn draw(&self, context: &Context) {
        if !self.visible {
            return;
        }
        
        context.save().expect("Failed to save context");
        
        // Apply transforms
        context.translate(self.position.x, self.position.y);
        
        // Create a matrix using the transform fields
        let matrix = self.transform.to_cairo_matrix();
        context.transform(matrix);
        
        // Build the path
        self.build_path(context);
        
        // Set fill and stroke styles
        match &self.fill {
            FillStyle::Solid(color) => {
                context.set_source_rgba(
                    color.r,
                    color.g,
                    color.b,
                    color.a
                );
            },
            FillStyle::Gradient(gradient) => {
                // Set up gradient pattern
                match &gradient.gradient_type {
                    GradientType::Linear { start, end } => {
                        let linear = cairo::LinearGradient::new(
                            start.x, start.y,
                            end.x, end.y
                        );
                        
                        for &(offset, ref color) in &gradient.stops {
                            linear.add_color_stop_rgba(
                                offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        
                        context.set_source(&linear);
                    },
                    GradientType::Radial { center, radius } => {
                        let radial = cairo::RadialGradient::new(
                            center.x, center.y, 0.0,
                            center.x, center.y, 
                            *radius
                        );
                        
                        for &(offset, ref color) in &gradient.stops {
                            radial.add_color_stop_rgba(
                                offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        
                        context.set_source(&radial);
                    },
                    GradientType::Conical { center: _, angle } => {
                        // Not directly supported in Cairo, fallback to a solid color
                        let default_color = Color::new(0.0, 0.0, 0.0, 1.0);
                        let color = gradient.stops.first().map(|(_, c)| c.clone()).unwrap_or(default_color);
                        context.set_source_rgba(
                            color.r,
                            color.g,
                            color.b,
                            color.a
                        );
                        return;
                    }
                };
            },
            FillStyle::Pattern(_) => {
                // Pattern fills not implemented yet
                context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            },
            FillStyle::None => {
                // No fill
                context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            }
        }
        
        // Create path based on shape type
        match &self.shape_type {
            ShapeType::Rectangle { width, height, corner_radius } => {
                if *corner_radius > 0.0 {
                    // Rounded rectangle
                    let x = self.position.x;
                    let y = self.position.y;
                    let w = *width;
                    let h = *height;
                    let r = corner_radius.min(w / 2.0).min(h / 2.0);
                    
                    // Top left corner
                    context.move_to(x + r, y);
                    // Top edge and top right corner
                    context.line_to(x + w - r, y);
                    context.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
                    // Right edge and bottom right corner
                    context.line_to(x + w, y + h - r);
                    context.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
                    // Bottom edge and bottom left corner
                    context.line_to(x + r, y + h);
                    context.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
                    // Left edge and top left corner
                    context.line_to(x, y + r);
                    context.arc(x + r, y + r, r, std::f64::consts::PI, -std::f64::consts::FRAC_PI_2);
                } else {
                    // Regular rectangle
                    context.rectangle(self.position.x, self.position.y, *width, *height);
                }
            },
            ShapeType::Ellipse { radius_x, radius_y } => {
                // Draw ellipse using a scaled circle
                context.save();
                context.translate(self.position.x, self.position.y);
                context.scale(*radius_x, *radius_y);
                context.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
                context.restore();
            },
            ShapeType::Circle { radius } => {
                context.arc(self.position.x, self.position.y, *radius, 0.0, 2.0 * std::f64::consts::PI);
            },
            ShapeType::Polygon { radius, sides } => {
                let sides = *sides as usize;
                if sides >= 3 {
                    let angle_step = 2.0 * std::f64::consts::PI / sides as f64;
                    
                    context.move_to(
                        self.position.x + radius * angle_step.cos(),
                        self.position.y + radius * angle_step.sin()
                    );
                    
                    for i in 1..=sides {
                        let angle = angle_step * i as f64;
                        context.line_to(
                            self.position.x + radius * angle.cos(),
                            self.position.y + radius * angle.sin()
                        );
                    }
                    
                    context.close_path();
                }
            },
            ShapeType::Star { outer_radius, inner_radius, points } => {
                let points = *points as usize;
                if points >= 3 {
                    let angle_step = std::f64::consts::PI / points as f64;
                    
                    // Start at the top point
                    context.move_to(
                        self.position.x,
                        self.position.y - outer_radius
                    );
                    
                    for i in 1..=points*2 {
                        let angle = angle_step * i as f64 - std::f64::consts::FRAC_PI_2;
                        let radius = if i % 2 == 1 { inner_radius } else { outer_radius };
                        
                        context.line_to(
                            self.position.x + radius * angle.cos(),
                            self.position.y + radius * angle.sin()
                        );
                    }
                    
                    context.close_path();
                }
            },
            ShapeType::Custom { path } => {
                // Move to position
                context.translate(self.position.x, self.position.y);
                
                // Draw the custom path
                path.draw(context);
            }
        }
        
        // Fill the shape if there's a fill style
        match &self.fill {
            FillStyle::Solid(color) => {
                context.set_source_rgba(
                    color.r,
                    color.g,
                    color.b,
                    color.a
                );
            },
            FillStyle::Gradient(gradient) => {
                match &gradient.gradient_type {
                    GradientType::Linear { start, end } => {
                        let linear = cairo::LinearGradient::new(
                            start.x, start.y,
                            end.x, end.y
                        );
                        
                        for (offset, ref color) in &gradient.stops {
                            linear.add_color_stop_rgba(
                                *offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        
                        context.set_source(&linear);
                    },
                    GradientType::Radial { center, radius } => {
                        let radial = cairo::RadialGradient::new(
                            center.x, center.y, 0.0,
                            center.x, center.y, 
                            *radius
                        );
                        
                        for (offset, ref color) in &gradient.stops {
                            radial.add_color_stop_rgba(
                                *offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        
                        context.set_source(&radial);
                    },
                    GradientType::Conical { center: _, angle } => {
                        // Not directly supported in Cairo, fallback to a solid color
                        let default_color = Color::new(0.0, 0.0, 0.0, 1.0);
                        let color = gradient.stops.first().map(|(_, c)| c.clone()).unwrap_or(default_color);
                        context.set_source_rgba(
                            color.r,
                            color.g,
                            color.b,
                            color.a
                        );
                        return;
                    }
                };
            },
            FillStyle::Pattern(_) => {
                // Pattern fills not implemented yet
                context.set_source_rgba(0.5, 0.5, 0.5, 1.0);
            },
            FillStyle::None => {
                // No fill
                context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            }
        }
        
        context.fill_preserve().expect("Failed to fill shape");
        
        // Draw stroke 
        context.set_line_width(self.stroke.width);
        
        // Set line cap
        match self.stroke.line_cap {
            LineCap::Butt => context.set_line_cap(cairo::LineCap::Butt),
            LineCap::Round => context.set_line_cap(cairo::LineCap::Round),
            LineCap::Square => context.set_line_cap(cairo::LineCap::Square),
            _ => context.set_line_cap(cairo::LineCap::Butt), // Default to Butt for any unknown values
        }
        
        // Set line join
        match self.stroke.line_join {
            LineJoin::Miter => context.set_line_join(cairo::LineJoin::Miter),
            LineJoin::Round => context.set_line_join(cairo::LineJoin::Round),
            LineJoin::Bevel => context.set_line_join(cairo::LineJoin::Bevel),
            _ => context.set_line_join(cairo::LineJoin::Miter), // Default to Miter for any unknown values
        }
        
        // Set dash pattern if available
        match self.stroke.line_dash {
            LineDash::Solid => {}, // No dash
            LineDash::Dashed => {
                context.set_dash(&[10.0, 5.0], 0.0);
            },
            LineDash::Dotted => {
                context.set_dash(&[2.0, 4.0], 0.0);
            },
            LineDash::DashDot => {
                context.set_dash(&[10.0, 5.0, 2.0, 5.0], 0.0);
            },
            LineDash::None => {
                // No stroke
            },
        }
        
        context.stroke().expect("Failed to stroke shape");
        
        // Draw selection handles or outline if selected
        if self.selection_state != SelectionState::None {
            let bounds = self.get_bounds();
            
            // Draw selection outline
            context.set_source_rgba(0.2, 0.6, 1.0, 0.8);
            context.set_line_width(1.0);
            context.set_dash(&[5.0, 5.0], 0.0);
            context.rectangle(bounds.x, bounds.y, bounds.width, bounds.height);
            context.stroke();
            
            // Draw selection handles
            context.set_dash(&[], 0.0);
            let handle_size = 5.0;
            
            // Draw handles at corners and edge midpoints
            let points = [
                (bounds.x, bounds.y),                              // Top-left
                (bounds.x + bounds.width / 2.0, bounds.y),         // Top-center
                (bounds.x + bounds.width, bounds.y),               // Top-right
                (bounds.x + bounds.width, bounds.y + bounds.height / 2.0), // Middle-right
                (bounds.x + bounds.width, bounds.y + bounds.height), // Bottom-right
                (bounds.x + bounds.width / 2.0, bounds.y + bounds.height), // Bottom-center
                (bounds.x, bounds.y + bounds.height),              // Bottom-left
                (bounds.x, bounds.y + bounds.height / 2.0),        // Middle-left
            ];
            
            for (x, y) in &points {
                context.rectangle(
                    x - handle_size / 2.0,
                    y - handle_size / 2.0,
                    handle_size,
                    handle_size
                );
                context.fill();
            }
        }
        
        context.restore().expect("Failed to restore context");
    }
}

// Add a method to Transform to convert it to a Cairo matrix
impl Transform {
    pub fn to_cairo_matrix(&self) -> cairo::Matrix {
        cairo::Matrix::new(
            self.a, self.b, self.c, self.d, self.e, self.f
        )
    }
}

impl VectorObject for Path {
    fn draw(&self, context: &Context) {
        // Move to the first point
        if let Some(first) = self.nodes.first() {
            context.move_to(first.point.position.x, first.point.position.y);
        }
        
        // Draw each node
        for i in 1..self.nodes.len() {
            let node = &self.nodes[i];
            match node.node_type {
                PathNodeType::Line => {
                    context.line_to(node.point.position.x, node.point.position.y);
                },
                PathNodeType::Curve => {
                    context.curve_to(
                        node.point.control_in.x, node.point.control_in.y,
                        node.point.control_out.x, node.point.control_out.y,
                        node.point.position.x, node.point.position.y
                    );
                }
            }
        }
        
        // Close the path if it's closed
        if self.closed {
            context.close_path();
        }
    }
    
    fn get_bounds(&self) -> Rect {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        
        for node in &self.nodes {
            min_x = min_x.min(node.point.position.x);
            min_y = min_y.min(node.point.position.y);
            max_x = max_x.max(node.point.position.x);
            max_y = max_y.max(node.point.position.y);
            
            // Control points are direct Point values, not Options
            min_x = min_x.min(node.point.control_in.x);
            min_y = min_y.min(node.point.control_in.y);
            max_x = max_x.max(node.point.control_in.x);
            max_y = max_y.max(node.point.control_in.y);
            
            min_x = min_x.min(node.point.control_out.x);
            min_y = min_y.min(node.point.control_out.y);
            max_x = max_x.max(node.point.control_out.x);
            max_y = max_y.max(node.point.control_out.y);
        }
        
        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y
        }
    }
    
    fn contains_point(&self, point: &Point) -> bool {
        // Create a temporary context to test point containment
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1)
            .expect("Failed to create temporary surface");
        let context = cairo::Context::new(&surface)
            .expect("Failed to create temporary context");
        
        // Draw the path
        self.draw(&context);
        
        // Test if the point is inside the path
        context.in_fill(point.x, point.y)
            .expect("Failed to check if point is in fill")
    }
    
    fn transform(&mut self, transform: &Transform) {
        for node in &mut self.nodes {
            let matrix = transform.to_cairo_matrix();
            let (x, y) = matrix.transform_point(node.point.position.x, node.point.position.y);
            node.point.position.x = x;
            node.point.position.y = y;
            
            // Control points are direct Point values, not Options
            let (x, y) = matrix.transform_point(node.point.control_in.x, node.point.control_in.y);
            node.point.control_in.x = x;
            node.point.control_in.y = y;
            
            let (x, y) = matrix.transform_point(node.point.control_out.x, node.point.control_out.y);
            node.point.control_out.x = x;
            node.point.control_out.y = y;
        }
    }
    
    fn clone_box(&self) -> Box<dyn VectorObject> {
        Box::new(self.clone())
    }
}

// Fix gradient stops
impl Fill {
    pub fn apply(&self, context: &Context) {
        match self {
            Fill::Solid(color) => {
                context.set_source_rgba(
                    color.r,
                    color.g,
                    color.b,
                    color.a
                );
            }
            Fill::Gradient(gradient) => {
                match &gradient.gradient_type {
                    GradientType::Linear { start, end } => {
                        let linear = cairo::LinearGradient::new(start.x, start.y, end.x, end.y);
                        for &(offset, color) in &gradient.stops {
                            linear.add_color_stop_rgba(
                                offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        context.set_source(&linear).expect("Failed to set gradient source");
                    }
                    GradientType::Radial { center, radius } => {
                        let radial = cairo::RadialGradient::new(
                            center.x, center.y, 0.0,
                            center.x, center.y, radius
                        );
                        for &(offset, color) in &gradient.stops {
                            radial.add_color_stop_rgba(
                                offset,
                                color.r,
                                color.g,
                                color.b,
                                color.a
                            );
                        }
                        context.set_source(&radial).expect("Failed to set gradient source");
                    }
                    GradientType::Conical { center: _, angle } => {
                        // TODO: Implement conical gradient
                        let default_color = Color::new(0.0, 0.0, 0.0, 1.0);
                        let color = gradient.stops.first().map(|(_, c)| c.clone()).unwrap_or(default_color);
                        context.set_source_rgba(
                            color.r,
                            color.g,
                            color.b,
                            color.a
                        );
                    }
                }
            }
            Fill::Pattern(_) => {
                // TODO: Implement pattern fill
                context.set_source_rgba(0.0, 0.0, 0.0, 1.0);
            }
            Fill::None => {
                context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
} 