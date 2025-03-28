use cairo::{Context, LineCap, LineJoin};
use uuid::Uuid;
use crate::vector::{Point, Rect, VectorObject, Transform, Color, SelectionState};

/// Represents a vector path for drawing
#[derive(Debug, Clone)]
pub struct VectorPath {
    points: Vec<PathCommand>,
    stroke_width: f64,
    stroke_color: (f64, f64, f64, f64), // RGBA
    fill_color: (f64, f64, f64, f64),   // RGBA
    stroke: bool,
    fill: bool,
    line_cap: LineCap,
    line_join: LineJoin,
    closed: bool,
}

/// Commands used to build a path
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64), // Control point 1, control point 2, end point
    ClosePath,
}

impl VectorPath {
    /// Create a new empty path
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            stroke_width: 1.0,
            stroke_color: (0.0, 0.0, 0.0, 1.0), // Black
            fill_color: (1.0, 1.0, 1.0, 0.0),   // Transparent white
            stroke: true,
            fill: false,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            closed: false,
        }
    }
    
    /// Move to a new point
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.points.push(PathCommand::MoveTo(x, y));
    }
    
    /// Draw a line to the specified point
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.points.push(PathCommand::LineTo(x, y));
    }
    
    /// Draw a curve to the specified end point using two control points
    pub fn curve_to(&mut self, 
        control1_x: f64, control1_y: f64, 
        control2_x: f64, control2_y: f64, 
        end_x: f64, end_y: f64) {
        self.points.push(PathCommand::CurveTo(
            control1_x, control1_y, 
            control2_x, control2_y, 
            end_x, end_y
        ));
    }
    
    /// Close the path
    pub fn close_path(&mut self) {
        self.points.push(PathCommand::ClosePath);
        self.closed = true;
    }
    
    /// Set the stroke width
    pub fn set_stroke_width(&mut self, width: f64) {
        self.stroke_width = width;
    }
    
    /// Set the stroke color
    pub fn set_stroke_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.stroke_color = (r, g, b, a);
    }
    
    /// Set the fill color
    pub fn set_fill_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.fill_color = (r, g, b, a);
    }
    
    /// Enable/disable stroke
    pub fn set_stroke(&mut self, enabled: bool) {
        self.stroke = enabled;
    }
    
    /// Enable/disable fill
    pub fn set_fill(&mut self, enabled: bool) {
        self.fill = enabled;
    }
    
    /// Set the line cap style
    pub fn set_line_cap(&mut self, cap: LineCap) {
        self.line_cap = cap;
    }
    
    /// Set the line join style
    pub fn set_line_join(&mut self, join: LineJoin) {
        self.line_join = join;
    }
    
    /// Draw the path to a Cairo context
    pub fn draw(&self, cr: &Context) {
        if self.points.is_empty() {
            return;
        }
        
        cr.save().expect("Failed to save context");
        
        // Create the path
        cr.new_path();
        
        for cmd in &self.points {
            match cmd {
                PathCommand::MoveTo(x, y) => {
                    cr.move_to(*x, *y);
                },
                PathCommand::LineTo(x, y) => {
                    cr.line_to(*x, *y);
                },
                PathCommand::CurveTo(cx1, cy1, cx2, cy2, x, y) => {
                    cr.curve_to(*cx1, *cy1, *cx2, *cy2, *x, *y);
                },
                PathCommand::ClosePath => {
                    cr.close_path();
                },
            }
        }
        
        // If the path is closed and we didn't explicitly close it
        if self.closed && !self.points.contains(&PathCommand::ClosePath) {
            cr.close_path();
        }
        
        // Set drawing properties
        cr.set_line_width(self.stroke_width);
        cr.set_line_cap(self.line_cap);
        cr.set_line_join(self.line_join);
        
        // Fill if enabled
        if self.fill {
            cr.set_source_rgba(
                self.fill_color.0,
                self.fill_color.1,
                self.fill_color.2,
                self.fill_color.3
            );
            
            if self.stroke {
                cr.fill_preserve().expect("Failed to fill path");
            } else {
                cr.fill().expect("Failed to fill path");
            }
        }
        
        // Stroke if enabled
        if self.stroke {
            cr.set_source_rgba(
                self.stroke_color.0,
                self.stroke_color.1,
                self.stroke_color.2,
                self.stroke_color.3
            );
            cr.stroke().expect("Failed to stroke path");
        }
        
        cr.restore().expect("Failed to restore context");
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathNodeType {
    Point,      // Regular point (sharp corner)
    Smooth,     // Smooth bezier point (handles aligned)
    Symmetric,  // Symmetric bezier point (handles equal length)
    Asymmetric, // Asymmetric bezier point (handles independent)
}

/// A bezier point with a position and two control points
#[derive(Debug, Clone, PartialEq)]
pub struct BezierPoint {
    pub position: Point,
    pub control_in: Point,  // Control point coming into this point
    pub control_out: Point, // Control point going out from this point
}

impl BezierPoint {
    pub fn new(x: f64, y: f64) -> Self {
        let position = Point::new(x, y);
        Self {
            position,
            control_in: position,  // Default to the same point (no curves)
            control_out: position, // Default to the same point (no curves)
        }
    }
    
    pub fn with_controls(position: Point, control_in: Point, control_out: Point) -> Self {
        Self {
            position,
            control_in,
            control_out,
        }
    }
    
    pub fn set_control_in(&mut self, x: f64, y: f64) {
        self.control_in = Point::new(x, y);
    }
    
    pub fn set_control_out(&mut self, x: f64, y: f64) {
        self.control_out = Point::new(x, y);
    }
    
    pub fn make_symmetric(&mut self) {
        // Calculate vector from position to control_out
        let dx = self.control_out.x - self.position.x;
        let dy = self.control_out.y - self.position.y;
        
        // Set control_in to the opposite direction with the same length
        self.control_in.x = self.position.x - dx;
        self.control_in.y = self.position.y - dy;
    }
    
    pub fn make_smooth(&mut self) {
        // Calculate angle from position to control_out
        let angle_out = (self.control_out.y - self.position.y)
            .atan2(self.control_out.x - self.position.x);
        
        // Calculate distance from position to control_in
        let dist_in = self.position.distance(&self.control_in);
        
        // Set control_in to the opposite direction but keep its length
        self.control_in.x = self.position.x - dist_in * angle_out.cos();
        self.control_in.y = self.position.y - dist_in * angle_out.sin();
    }
}

/// A node in a path
#[derive(Debug, Clone, PartialEq)]
pub struct PathNode {
    pub point: BezierPoint,
    pub node_type: PathNodeType,
    pub selected: bool,
}

impl PathNode {
    pub fn new(x: f64, y: f64, node_type: PathNodeType) -> Self {
        Self {
            point: BezierPoint::new(x, y),
            node_type,
            selected: false,
        }
    }
    
    pub fn with_bezier(point: BezierPoint, node_type: PathNodeType) -> Self {
        Self {
            point,
            node_type,
            selected: false,
        }
    }
    
    pub fn set_type(&mut self, node_type: PathNodeType) {
        self.node_type = node_type;
        
        // Adjust control points based on the new type
        match node_type {
            PathNodeType::Point => {
                // Set control points to the position (no curves)
                self.point.control_in = self.point.position;
                self.point.control_out = self.point.position;
            },
            PathNodeType::Smooth => {
                self.point.make_smooth();
            },
            PathNodeType::Symmetric => {
                self.point.make_symmetric();
            },
            PathNodeType::Asymmetric => {
                // No adjustment needed, control points can be independent
            },
        }
    }
    
    pub fn select(&mut self, selected: bool) {
        self.selected = selected;
    }
}

/// A vector path consisting of multiple nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub id: String,
    pub nodes: Vec<PathNode>,
    pub closed: bool,
    pub bounds: Option<Rect>, // Cached bounds
}

impl Path {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            nodes: Vec::new(),
            closed: false,
            bounds: None,
        }
    }
    
    pub fn add_point(&mut self, x: f64, y: f64, node_type: PathNodeType) {
        let node = PathNode::new(x, y, node_type);
        self.nodes.push(node);
        self.bounds = None; // Invalidate cached bounds
    }
    
    pub fn add_node(&mut self, node: PathNode) {
        self.nodes.push(node);
        self.bounds = None; // Invalidate cached bounds
    }
    
    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
    }
    
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.bounds = None;
    }
    
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn get_node(&self, index: usize) -> Option<&PathNode> {
        self.nodes.get(index)
    }
    
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut PathNode> {
        self.nodes.get_mut(index)
    }
    
    pub fn remove_node(&mut self, index: usize) {
        if index < self.nodes.len() {
            self.nodes.remove(index);
            self.bounds = None; // Invalidate cached bounds
        }
    }
    
    pub fn insert_node(&mut self, index: usize, node: PathNode) {
        if index <= self.nodes.len() {
            self.nodes.insert(index, node);
            self.bounds = None; // Invalidate cached bounds
        }
    }
    
    pub fn get_selected_nodes(&self) -> Vec<usize> {
        self.nodes.iter()
            .enumerate()
            .filter(|(_, node)| node.selected)
            .map(|(i, _)| i)
            .collect()
    }
    
    pub fn select_all_nodes(&mut self, selected: bool) {
        for node in &mut self.nodes {
            node.selected = selected;
        }
    }
    
    pub fn get_bounds(&self) -> Rect {
        if let Some(bounds) = self.bounds {
            return bounds;
        }
        
        if self.nodes.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }
        
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        
        for node in &self.nodes {
            // Consider both position and control points for bounds
            for point in [&node.point.position, &node.point.control_in, &node.point.control_out] {
                min_x = min_x.min(point.x);
                min_y = min_y.min(point.y);
                max_x = max_x.max(point.x);
                max_y = max_y.max(point.y);
            }
        }
        
        let bounds = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
        let mut path = self.clone();
        path.bounds = Some(bounds);
        
        bounds
    }
    
    pub fn transform(&mut self, transform: &Transform) {
        for node in &mut self.nodes {
            node.point.position = transform.apply_to_point(&node.point.position);
            node.point.control_in = transform.apply_to_point(&node.point.control_in);
            node.point.control_out = transform.apply_to_point(&node.point.control_out);
        }
        
        self.bounds = None; // Invalidate cached bounds
    }
    
    pub fn build_path(&self, context: &Context) {
        if self.nodes.is_empty() {
            return;
        }
        
        context.new_path();
        
        // Start path at the first node
        let first = &self.nodes[0];
        context.move_to(first.point.position.x, first.point.position.y);
        
        // Draw bezier segments between nodes
        for i in 1..self.nodes.len() {
            let prev = &self.nodes[i - 1];
            let current = &self.nodes[i];
            
            // Check if we need a curve or straight line
            if prev.point.control_out != prev.point.position || current.point.control_in != current.point.position {
                // Draw cubic bezier curve
                context.curve_to(
                    prev.point.control_out.x, prev.point.control_out.y,
                    current.point.control_in.x, current.point.control_in.y,
                    current.point.position.x, current.point.position.y
                );
            } else {
                // Draw straight line
                context.line_to(current.point.position.x, current.point.position.y);
            }
        }
        
        // Close the path if needed
        if self.closed {
            if self.nodes.len() > 2 {
                let last = &self.nodes[self.nodes.len() - 1];
                let first = &self.nodes[0];
                
                // Check if we need a curve or straight line for closing the path
                if last.point.control_out != last.point.position || first.point.control_in != first.point.position {
                    // Draw cubic bezier curve
                    context.curve_to(
                        last.point.control_out.x, last.point.control_out.y,
                        first.point.control_in.x, first.point.control_in.y,
                        first.point.position.x, first.point.position.y
                    );
                } else {
                    // Close with straight line
                    context.close_path();
                }
            } else {
                context.close_path();
            }
        }
    }
    
    pub fn draw_nodes(&self, context: &Context) {
        for (i, node) in self.nodes.iter().enumerate() {
            // Draw control lines
            if node.point.control_in != node.point.position || node.point.control_out != node.point.position {
                context.set_source_rgba(0.5, 0.5, 0.5, 0.7);
                context.set_line_width(0.5);
                
                // Line from control_in to position
                if node.point.control_in != node.point.position {
                    context.move_to(node.point.control_in.x, node.point.control_in.y);
                    context.line_to(node.point.position.x, node.point.position.y);
                    context.stroke();
                }
                
                // Line from position to control_out
                if node.point.control_out != node.point.position {
                    context.move_to(node.point.position.x, node.point.position.y);
                    context.line_to(node.point.control_out.x, node.point.control_out.y);
                    context.stroke();
                }
            }
            
            // Draw the node points
            // Main point
            if node.selected {
                context.set_source_rgba(0.0, 0.7, 1.0, 1.0); // Blue for selected
            } else {
                context.set_source_rgba(1.0, 1.0, 1.0, 1.0); // White for unselected
            }
            
            // Node shape depends on type
            match node.node_type {
                PathNodeType::Point => {
                    // Square for corner points
                    context.rectangle(
                        node.point.position.x - 3.0,
                        node.point.position.y - 3.0,
                        6.0,
                        6.0
                    );
                },
                PathNodeType::Smooth | PathNodeType::Symmetric => {
                    // Circle for smooth points
                    context.arc(
                        node.point.position.x,
                        node.point.position.y,
                        3.0,
                        0.0,
                        std::f64::consts::PI * 2.0
                    );
                },
                PathNodeType::Asymmetric => {
                    // Diamond for asymmetric points
                    context.move_to(node.point.position.x, node.point.position.y - 3.0);
                    context.line_to(node.point.position.x + 3.0, node.point.position.y);
                    context.line_to(node.point.position.x, node.point.position.y + 3.0);
                    context.line_to(node.point.position.x - 3.0, node.point.position.y);
                    context.close_path();
                },
            }
            
            context.fill();
            
            // Draw the control points if they're not coincident with the main point
            context.set_source_rgba(0.8, 0.2, 0.2, 0.8); // Red for control points
            
            if node.point.control_in != node.point.position {
                context.arc(
                    node.point.control_in.x,
                    node.point.control_in.y,
                    2.0,
                    0.0,
                    std::f64::consts::PI * 2.0
                );
                context.fill();
            }
            
            if node.point.control_out != node.point.position {
                context.arc(
                    node.point.control_out.x,
                    node.point.control_out.y,
                    2.0,
                    0.0,
                    std::f64::consts::PI * 2.0
                );
                context.fill();
            }
            
            // Draw index number for debugging
            context.set_source_rgba(0.8, 0.8, 0.2, 0.9);
            context.set_font_size(10.0);
            context.move_to(node.point.position.x + 5.0, node.point.position.y - 5.0);
            context.show_text(&i.to_string()).unwrap();
        }
    }
} 