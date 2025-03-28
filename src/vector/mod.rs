use cairo::{Context, Matrix};
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::collections::HashMap;
use gtk4::cairo::{LineCap, LineJoin, Path, Pattern};
use gtk4::gdk::RGBA;
use std::any::Any;
use log::{debug, error, info, trace, warn};
use crate::core::Canvas;

/// A vector path consisting of multiple path segments
#[derive(Clone, Debug)]
pub struct VectorPath {
    segments: Vec<PathSegment>,
    is_closed: bool,
    stroke_width: f64,
    stroke_color: (f64, f64, f64, f64),
    fill_color: Option<(f64, f64, f64, f64)>,
}

/// A segment of a vector path
#[derive(Clone, Debug)]
pub enum PathSegment {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64), // Cubic Bezier: control1_x, control1_y, control2_x, control2_y, end_x, end_y
    QuadraticTo(f64, f64, f64, f64),       // Quadratic Bezier: control_x, control_y, end_x, end_y
    ArcTo(f64, f64, f64, bool, bool, f64, f64), // Arc: radius_x, radius_y, rotation, large_arc, sweep, end_x, end_y
    Close,
}

impl VectorPath {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            is_closed: false,
            stroke_width: 1.0,
            stroke_color: (0.0, 0.0, 0.0, 1.0),
            fill_color: None,
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.segments.push(PathSegment::MoveTo(x, y));
        self
    }

    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.segments.push(PathSegment::LineTo(x, y));
        self
    }

    pub fn curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> &mut Self {
        self.segments.push(PathSegment::CurveTo(x1, y1, x2, y2, x3, y3));
        self
    }

    pub fn quadratic_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> &mut Self {
        self.segments.push(PathSegment::QuadraticTo(x1, y1, x2, y2));
        self
    }

    pub fn arc_to(&mut self, rx: f64, ry: f64, rotation: f64, large_arc: bool, sweep: bool, x: f64, y: f64) -> &mut Self {
        self.segments.push(PathSegment::ArcTo(rx, ry, rotation, large_arc, sweep, x, y));
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.segments.push(PathSegment::Close);
        self.is_closed = true;
        self
    }

    pub fn set_stroke_width(&mut self, width: f64) -> &mut Self {
        self.stroke_width = width;
        self
    }

    pub fn set_stroke_color(&mut self, r: f64, g: f64, b: f64, a: f64) -> &mut Self {
        self.stroke_color = (r, g, b, a);
        self
    }

    pub fn set_fill_color(&mut self, r: f64, g: f64, b: f64, a: f64) -> &mut Self {
        self.fill_color = Some((r, g, b, a));
        self
    }

    pub fn clear_fill(&mut self) -> &mut Self {
        self.fill_color = None;
        self
    }

    pub fn draw(&self, cr: &Context) {
        cr.save().expect("Failed to save Cairo context");
        
        // Begin a new path
        cr.new_path();
        
        // Add all segments to the path
        for segment in &self.segments {
            match segment {
                PathSegment::MoveTo(x, y) => cr.move_to(*x, *y),
                PathSegment::LineTo(x, y) => cr.line_to(*x, *y),
                PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => cr.curve_to(*x1, *y1, *x2, *y2, *x3, *y3),
                PathSegment::QuadraticTo(x1, y1, x2, y2) => {
                    // Convert quadratic to cubic bezier
                    if let Ok((last_x, last_y)) = cr.current_point() {
                        let cx1 = last_x + 2.0/3.0 * (*x1 - last_x);
                        let cy1 = last_y + 2.0/3.0 * (*y1 - last_y);
                        let cx2 = *x2 + 2.0/3.0 * (*x1 - *x2);
                        let cy2 = *y2 + 2.0/3.0 * (*y1 - *y2);
                        cr.curve_to(cx1, cy1, cx2, cy2, *x2, *y2);
                    }
                }
                PathSegment::ArcTo(rx, ry, angle, large_arc, sweep, x, y) => {
                    // SVG-style arc implementation
                    if let Ok((x0, y0)) = cr.current_point() {
                        let points = arc_to_bezier(x0, y0, *rx, *ry, *angle, *large_arc, *sweep, *x, *y);
                        for i in (0..points.len()).step_by(6) {
                            if i + 5 < points.len() {
                                cr.curve_to(
                                    points[i], points[i+1], 
                                    points[i+2], points[i+3], 
                                    points[i+4], points[i+5]
                                );
                            }
                        }
                    }
                }
                PathSegment::Close => cr.close_path(),
            }
        }
        
        // Fill if a fill color is set
        if let Some((r, g, b, a)) = self.fill_color {
            cr.set_source_rgba(r, g, b, a);
            if self.stroke_width > 0.0 {
                cr.fill_preserve().expect("Failed to fill path");
            } else {
                cr.fill().expect("Failed to fill path");
            }
        }
        
        // Stroke the path
        if self.stroke_width > 0.0 {
            let (r, g, b, a) = self.stroke_color;
            cr.set_source_rgba(r, g, b, a);
            cr.set_line_width(self.stroke_width);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.set_line_join(cairo::LineJoin::Round);
            cr.stroke().expect("Failed to stroke path");
        }
        
        cr.restore().expect("Failed to restore Cairo context");
    }
}

/// A vectorized shape
#[derive(Clone, Debug)]
pub enum VectorShape {
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        rx: f64,
        ry: f64,
    },
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
    },
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
    },
    Polygon {
        points: Vec<(f64, f64)>,
    },
    Path {
        path: VectorPath,
    },
    Group {
        shapes: Vec<VectorShape>,
    },
}

impl VectorShape {
    pub fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::Rectangle { x, y, width, height, rx: 0.0, ry: 0.0 }
    }
    
    pub fn rounded_rectangle(x: f64, y: f64, width: f64, height: f64, rx: f64, ry: f64) -> Self {
        Self::Rectangle { x, y, width, height, rx, ry }
    }
    
    pub fn circle(cx: f64, cy: f64, r: f64) -> Self {
        Self::Circle { cx, cy, r }
    }
    
    pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> Self {
        Self::Ellipse { cx, cy, rx, ry }
    }
    
    pub fn polygon(points: Vec<(f64, f64)>) -> Self {
        Self::Polygon { points }
    }
    
    pub fn to_path(&self) -> VectorPath {
        let mut path = VectorPath::new();
        
        match self {
            Self::Rectangle { x, y, width, height, rx, ry } => {
                if *rx <= 0.0 || *ry <= 0.0 {
                    // Simple rectangle
                    path.move_to(*x, *y)
                        .line_to(x + width, *y)
                        .line_to(x + width, y + height)
                        .line_to(*x, y + height)
                        .close();
                } else {
                    // Rounded rectangle
                    let rx = rx.min(width / 2.0);
                    let ry = ry.min(height / 2.0);
                    
                    path.move_to(x + rx, *y)
                        .line_to(x + width - rx, *y)
                        .arc_to(rx, ry, 0.0, false, true, x + width, y + ry)
                        .line_to(x + width, y + height - ry)
                        .arc_to(rx, ry, 0.0, false, true, x + width - rx, y + height)
                        .line_to(x + rx, y + height)
                        .arc_to(rx, ry, 0.0, false, true, *x, y + height - ry)
                        .line_to(*x, y + ry)
                        .arc_to(rx, ry, 0.0, false, true, x + rx, *y)
                        .close();
                }
            },
            Self::Circle { cx, cy, r } => {
                // Approximate circle with bezier curves
                let c = 0.551915024494; // magic number to approximate circle with beziers
                path.move_to(cx + r, *cy)
                    .curve_to(cx + r, cy + c * r, cx + c * r, cy + r, *cx, cy + r)
                    .curve_to(cx - c * r, cy + r, cx - r, cy + c * r, cx - r, *cy)
                    .curve_to(cx - r, cy - c * r, cx - c * r, cy - r, *cx, cy - r)
                    .curve_to(cx + c * r, cy - r, cx + r, cy - c * r, cx + r, *cy)
                    .close();
            },
            Self::Ellipse { cx, cy, rx, ry } => {
                // Approximate ellipse with bezier curves
                let c = 0.551915024494; // magic number to approximate ellipse with beziers
                path.move_to(cx + rx, *cy)
                    .curve_to(cx + rx, cy + c * ry, cx + c * rx, cy + ry, *cx, cy + ry)
                    .curve_to(cx - c * rx, cy + ry, cx - rx, cy + c * ry, cx - rx, *cy)
                    .curve_to(cx - rx, cy - c * ry, cx - c * rx, cy - ry, *cx, cy - ry)
                    .curve_to(cx + c * rx, cy - ry, cx + rx, cy - c * ry, cx + rx, *cy)
                    .close();
            },
            Self::Polygon { points } => {
                if !points.is_empty() {
                    path.move_to(points[0].0, points[0].1);
                    for (x, y) in points.iter().skip(1) {
                        path.line_to(*x, *y);
                    }
                    path.close();
                }
            },
            Self::Path { path: p } => {
                return p.clone();
            },
            Self::Group { shapes } => {
                // For a group, we combine all paths
                for shape in shapes {
                    let shape_path = shape.to_path();
                    for segment in shape_path.segments {
                        match segment {
                            PathSegment::MoveTo(x, y) => path.move_to(x, y),
                            PathSegment::LineTo(x, y) => path.line_to(x, y),
                            PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => path.curve_to(x1, y1, x2, y2, x3, y3),
                            PathSegment::QuadraticTo(x1, y1, x2, y2) => path.quadratic_to(x1, y1, x2, y2),
                            PathSegment::ArcTo(rx, ry, angle, large_arc, sweep, x, y) => 
                                path.arc_to(rx, ry, angle, large_arc, sweep, x, y),
                            PathSegment::Close => path.close(),
                        };
                    }
                }
            },
        }
        
        path
    }
    
    pub fn draw(&self, cr: &Context) {
        match self {
            Self::Path { path } => {
                path.draw(cr);
            },
            _ => {
                let path = self.to_path();
                path.draw(cr);
            }
        }
    }
}

impl Default for VectorShape {
    fn default() -> Self {
        VectorShape::rectangle(0.0, 0.0, 10.0, 10.0)
    }
}

/// Represents a vector layer containing multiple vector objects/shapes
#[derive(Clone)]
pub struct VectorLayer {
    shapes: Vec<VectorShape>,
    name: String,
    visible: bool,
    opacity: f64,
}

impl VectorLayer {
    pub fn new(name: &str) -> Self {
        Self {
            shapes: Vec::new(),
            name: name.to_string(),
            visible: true,
            opacity: 1.0,
        }
    }
    
    pub fn add_shape(&mut self, shape: VectorShape) {
        self.shapes.push(shape);
    }
    
    pub fn draw(&self, cr: &Context) {
        if !self.visible {
            return;
        }
        
        cr.save().expect("Failed to save Cairo context");
        cr.set_operator(cairo::Operator::Over);
        
        // Apply layer opacity if it's not 1.0
        if self.opacity < 1.0 {
            // Get size for the temporary surface
            let (width, height) = get_size(cr);
            
            let temp_surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32, 
                width as i32, 
                height as i32
            ).expect("Failed to create temporary surface");
            
            let temp_cr = Context::new(&temp_surface).expect("Failed to create temporary context");
            
            // Draw all shapes to the temporary surface
            for shape in &self.shapes {
                shape.draw(&temp_cr);
            }
            
            // Draw the temporary surface with opacity
            cr.set_source_surface(&temp_surface, 0.0, 0.0).expect("Failed to set source surface");
            cr.paint_with_alpha(self.opacity).expect("Failed to paint with alpha");
        } else {
            // Draw all shapes directly
            for shape in &self.shapes {
                shape.draw(cr);
            }
        }
        
        cr.restore().expect("Failed to restore Cairo context");
    }
    
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

/// A vector document containing multiple vector layers
#[derive(Clone)]
pub struct VectorDocument {
    layers: Vec<VectorLayer>,
    active_layer: usize,
    width: i32,
    height: i32,
}

impl VectorDocument {
    pub fn new(width: i32, height: i32) -> Self {
        let mut doc = Self {
            layers: Vec::new(),
            active_layer: 0,
            width,
            height,
        };
        
        // Add a default layer
        doc.add_layer("Background");
        
        doc
    }
    
    pub fn add_layer(&mut self, name: &str) -> usize {
        self.layers.push(VectorLayer::new(name));
        let index = self.layers.len() - 1;
        self.active_layer = index;
        index
    }
    
    pub fn remove_layer(&mut self, index: usize) -> bool {
        if index < self.layers.len() {
            self.layers.remove(index);
            if self.active_layer >= self.layers.len() {
                self.active_layer = self.layers.len().saturating_sub(1);
            }
            true
        } else {
            false
        }
    }
    
    pub fn set_active_layer(&mut self, index: usize) -> bool {
        if index < self.layers.len() {
            self.active_layer = index;
            true
        } else {
            false
        }
    }
    
    pub fn get_active_layer(&self) -> Option<&VectorLayer> {
        self.layers.get(self.active_layer)
    }
    
    pub fn get_active_layer_mut(&mut self) -> Option<&mut VectorLayer> {
        self.layers.get_mut(self.active_layer)
    }
    
    pub fn get_layers(&self) -> &[VectorLayer] {
        &self.layers
    }
    
    pub fn get_layers_mut(&mut self) -> &mut [VectorLayer] {
        &mut self.layers
    }
    
    pub fn draw(&self, cr: &Context) {
        cr.save().expect("Failed to save Cairo context");
        
        // Clear the surface
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().expect("Failed to paint background");
        
        // Draw all visible layers
        for layer in &self.layers {
            layer.draw(cr);
        }
        
        cr.restore().expect("Failed to restore Cairo context");
    }
    
    pub fn width(&self) -> i32 {
        self.width
    }
    
    pub fn height(&self) -> i32 {
        self.height
    }
    
    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }
    
    pub fn add_shape(&mut self, shape: VectorShape) -> bool {
        if let Some(layer) = self.get_active_layer_mut() {
            layer.add_shape(shape);
            true
        } else {
            false
        }
    }
}

// Helper function to convert SVG-style arc to bezier curves
fn arc_to_bezier(x1: f64, y1: f64, rx: f64, ry: f64, angle: f64, large_arc: bool, sweep: bool, x2: f64, y2: f64) -> Vec<f64> {
    // Implementation based on SVG spec for approximating arcs with bezier curves
    // This is a simplified version - a full implementation would be more complex
    
    // Convert angles to radians
    let angle_rad = angle * PI / 180.0;
    
    // Ensure radii are positive
    let rx = rx.abs();
    let ry = ry.abs();
    
    // If the arc is empty, connect with a straight line
    if x1 == x2 && y1 == y2 {
        return vec![];
    }
    
    // Step 1: Transform to origin
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    
    // Step 2: Apply angle rotation to get to ellipse coordinate system
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();
    let x1p = cos_angle * dx + sin_angle * dy;
    let y1p = -sin_angle * dx + cos_angle * dy;
    
    // Step 3: Make sure radii are large enough
    let lambda = (x1p / rx).powi(2) + (y1p / ry).powi(2);
    let mut rx_scaled = rx;
    let mut ry_scaled = ry;
    
    if lambda > 1.0 {
        let sqrt_lambda = lambda.sqrt();
        rx_scaled *= sqrt_lambda;
        ry_scaled *= sqrt_lambda;
    }
    
    // Step 4: Compute center parameters
    let rxsq = rx_scaled.powi(2);
    let rysq = ry_scaled.powi(2);
    let x1psq = x1p.powi(2);
    let y1psq = y1p.powi(2);
    
    let term = rxsq * rysq - rxsq * y1psq - rysq * x1psq;
    let term_sqrt = if term <= 0.0 { 0.0 } else { (term / (rxsq * y1psq + rysq * x1psq)).sqrt() };
    
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let cxp = sign * term_sqrt * rx_scaled * y1p / ry_scaled;
    let cyp = sign * -term_sqrt * ry_scaled * x1p / rx_scaled;
    
    // Step 5: Transform back to the original coordinate system
    let cx = cos_angle * cxp - sin_angle * cyp + (x1 + x2) / 2.0;
    let cy = sin_angle * cxp + cos_angle * cyp + (y1 + y2) / 2.0;
    
    // Step 6: Compute the angle sweep
    let ux = (x1p - cxp) / rx_scaled;
    let uy = (y1p - cyp) / ry_scaled;
    let vx = (-x1p - cxp) / rx_scaled;
    let vy = (-y1p - cyp) / ry_scaled;
    
    let start_angle = vector_angle(1.0, 0.0, ux, uy);
    let mut delta_angle = vector_angle(ux, uy, vx, vy);
    
    if !sweep && delta_angle > 0.0 {
        delta_angle -= 2.0 * PI;
    } else if sweep && delta_angle < 0.0 {
        delta_angle += 2.0 * PI;
    }
    
    // Step 7: Approximate the arc with bezier curves
    let segments = ((delta_angle.abs() / (PI / 2.0)).ceil() as usize).max(1);
    let delta_per_segment = delta_angle / segments as f64;
    let segment_angle = 4.0 * (delta_per_segment / 2.0).tan() / 3.0;
    
    let mut result = Vec::new();
    
    let mut current_angle = start_angle;
    let mut t = Matrix::new(
        cos_angle, sin_angle,
        -sin_angle, cos_angle,
        cx, cy
    );
    
    for i in 0..segments {
        let angle = current_angle + delta_per_segment;
        
        let p1x = cx + rx_scaled * current_angle.cos();
        let p1y = cy + ry_scaled * current_angle.sin();
        
        let p2x = cx + rx_scaled * angle.cos();
        let p2y = cy + ry_scaled * angle.sin();
        
        let tan = segment_angle;
        
        let c1x = p1x - tan * rx_scaled * current_angle.sin();
        let c1y = p1y + tan * ry_scaled * current_angle.cos();
        
        let c2x = p2x + tan * rx_scaled * angle.sin();
        let c2y = p2y - tan * ry_scaled * angle.cos();
        
        // Transform the control points back
        let c1_transformed = transform_point(c1x, c1y, &t);
        let c2_transformed = transform_point(c2x, c2y, &t);
        let p2_transformed = transform_point(p2x, p2y, &t);
        
        // Add to result
        result.push(c1_transformed.0);
        result.push(c1_transformed.1);
        result.push(c2_transformed.0);
        result.push(c2_transformed.1);
        result.push(p2_transformed.0);
        result.push(p2_transformed.1);
        
        current_angle = angle;
    }
    
    result
}

fn vector_angle(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
    let dot = ux * vx + uy * vy;
    let len = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
    
    let angle = if len != 0.0 {
        (dot / len).clamp(-1.0, 1.0).acos()
    } else {
        0.0
    };
    
    if ux * vy - uy * vx < 0.0 {
        -angle
    } else {
        angle
    }
}

fn transform_point(x: f64, y: f64, matrix: &cairo::Matrix) -> (f64, f64) {
    let (xt, yt) = matrix.transform_point(x, y);
    (xt, yt)
}

pub fn get_size(cr: &Context) -> (f64, f64) {
    let mut x1 = 0.0;
    let mut y1 = 0.0;
    let mut x2 = 0.0;
    let mut y2 = 0.0;
    
    if let Ok((x1_val, y1_val, x2_val, y2_val)) = cr.clip_extents() {
        x1 = x1_val;
        y1 = y1_val;
        x2 = x2_val;
        y2 = y2_val;
        ((x2 - x1).abs(), (y2 - y1).abs())
    } else {
        (800.0, 600.0)
    }
}

fn draw_line_dash(cr: &Context, dash: &LineDash) {
    match dash {
        LineDash::None => cr.set_dash(&[], 0.0),
        LineDash::Solid => cr.set_dash(&[1.0], 0.0),
        LineDash::Dashed => cr.set_dash(&[6.0, 2.0], 0.0),
        LineDash::Dotted => cr.set_dash(&[2.0, 2.0], 0.0),
        LineDash::DashDot => cr.set_dash(&[6.0, 2.0, 2.0, 2.0], 0.0),
    }
}

// Vector module - Vector graphics support

pub mod shape;
pub mod path;
pub mod text;
pub mod document;

pub use self::shape::{VectorShape as ShapeImpl, ShapeType, FillStyle, StrokeStyle, Gradient, GradientType, Color, LineDash};
pub use self::path::{PathNode, PathNodeType, BezierPoint};
pub use self::text::{TextShape, TextStyle, TextAlignment, FontWeight, FontStyle};
pub use self::document::{VectorDocument as DocumentImpl, VectorLayer as LayerImpl};

// Basic structures

/// Point in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        trace!("Creating Point({}, {})", x, y);
        Self { x, y }
    }
    
    pub fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dist = (dx * dx + dy * dy).sqrt();
        trace!("Distance between points: {:.2}", dist);
        dist
    }
    
    // Alias for distance for compatibility
    pub fn distance_to(&self, other: &Point) -> f64 {
        self.distance(other)
    }
    
    pub fn lerp(&self, other: &Point, t: f64) -> Self {
        let t_clamped = t.max(0.0).min(1.0);
        trace!("Interpolating points with t={:.2}", t_clamped);
        Self {
            x: self.x + (other.x - self.x) * t_clamped,
            y: self.y + (other.y - self.y) * t_clamped,
        }
    }
}

/// Rectangle in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        debug!("Creating Rect({}, {}, {}, {})", x, y, width, height);
        Self { x, y, width, height }
    }
    
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).abs();
        let height = (p1.y - p2.y).abs();
        
        debug!("Creating Rect from points ({}, {}) and ({}, {}): ({}, {}, {}, {})",
               p1.x, p1.y, p2.x, p2.y, x, y, width, height);
        
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, point: &Point) -> bool {
        let result = point.x >= self.x && 
                     point.x < self.x + self.width && 
                     point.y >= self.y && 
                     point.y < self.y + self.height;
        trace!("Point ({}, {}) contained in rect: {}", point.x, point.y, result);
        result
    }
    
    pub fn center(&self) -> Point {
        let center = Point::new(
            self.x + self.width / 2.0,
            self.y + self.height / 2.0
        );
        trace!("Rect center: ({}, {})", center.x, center.y);
        center
    }
    
    pub fn top_left(&self) -> Point {
        Point::new(self.x, self.y)
    }
    
    pub fn bottom_right(&self) -> Point {
        Point::new(self.x + self.width, self.y + self.height)
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        let result = !(other.x > self.x + self.width ||
                       other.x + other.width < self.x ||
                       other.y > self.y + self.height ||
                       other.y + other.height < self.y);
        trace!("Rect intersection check: {}", result);
        result
    }
    
    pub fn union(&self, other: &Rect) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x + self.width).max(other.x + other.width);
        let bottom = (self.y + self.height).max(other.y + other.height);
        let width = right - x;
        let height = bottom - y;
        
        debug!("Rect union: ({}, {}, {}, {})", x, y, width, height);
        Self { x, y, width, height }
    }
}

/// Affine transformation matrix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    // Row-major 3x3 matrix:
    // [a c e]
    // [b d f]
    // [0 0 1]
    pub a: f64, pub c: f64, pub e: f64,
    pub b: f64, pub d: f64, pub f: f64,
}

impl Transform {
    pub fn identity() -> Self {
        trace!("Creating identity transform");
        Self {
            a: 1.0, c: 0.0, e: 0.0,
            b: 0.0, d: 1.0, f: 0.0,
        }
    }
    
    pub fn translation(tx: f64, ty: f64) -> Self {
        debug!("Creating translation transform: ({}, {})", tx, ty);
        Self {
            a: 1.0, c: 0.0, e: tx,
            b: 0.0, d: 1.0, f: ty,
        }
    }
    
    pub fn scale(sx: f64, sy: f64) -> Self {
        debug!("Creating scale transform: ({}, {})", sx, sy);
        Self {
            a: sx, c: 0.0, e: 0.0,
            b: 0.0, d: sy, f: 0.0,
        }
    }
    
    pub fn rotation(angle_degrees: f64) -> Self {
        let angle_radians = angle_degrees * PI / 180.0;
        let s = angle_radians.sin();
        let c = angle_radians.cos();
        
        debug!("Creating rotation transform: {} degrees", angle_degrees);
        
        Self {
            a: c, c: -s, e: 0.0,
            b: s, d: c,  f: 0.0,
        }
    }
    
    pub fn multiply(&self, other: &Transform) -> Self {
        debug!("Multiplying transforms");
        Self {
            a: self.a * other.a + self.c * other.b,
            c: self.a * other.c + self.c * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            
            b: self.b * other.a + self.d * other.b,
            d: self.b * other.c + self.d * other.d,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }
    
    pub fn apply_to_point(&self, point: &Point) -> Point {
        let x = self.a * point.x + self.c * point.y + self.e;
        let y = self.b * point.x + self.d * point.y + self.f;
        trace!("Transform applied to point ({}, {}): result = ({}, {})", 
               point.x, point.y, x, y);
        Point::new(x, y)
    }
    
    pub fn invert(&self) -> Option<Self> {
        // Calculate determinant
        let det = self.a * self.d - self.b * self.c;
        
        if det.abs() < 1e-10 {
            warn!("Cannot invert transform: determinant is too small ({})", det);
            return None;
        }
        
        let inv_det = 1.0 / det;
        
        // Calculate matrix of minors and cofactors
        let result = Self {
            a: self.d * inv_det,
            c: -self.c * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            
            b: -self.b * inv_det,
            d: self.a * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        };
        
        debug!("Transform inverted successfully");
        Some(result)
    }
}

/// Selection state for a vector object
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionState {
    None,
    Selected,
    GroupSelected, // Selected as part of a group
    EditPoints,    // Node editing mode
}

/// Vector enums and types

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathOperation {
    None,
    Union,
    Subtract,
    Intersect,
    XOR,
    Divide,
}

/// Trait for downcasting objects to concrete types
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Implement AsAny for any type that can be converted to Any
impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Base trait for all vector objects
pub trait VectorObject: AsAny + Send + Sync + std::fmt::Debug {
    /// Get the bounding rectangle of the object
    fn get_bounds(&self) -> Rect;
    
    /// Check if the object contains a point
    fn contains_point(&self, point: &Point) -> bool;
    
    /// Apply a transformation to the object
    fn transform(&mut self, transform: &Transform);
    
    /// Draw the object to a Cairo context
    fn draw(&self, context: &Context);
    
    /// Clone the object
    fn clone_box(&self) -> Box<dyn VectorObject>;
}

impl Clone for Box<dyn VectorObject> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Re-exports from the module
// This allows us to use vector::{VectorShape, VectorPath, etc.} in other modules 