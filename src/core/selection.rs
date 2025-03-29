use image::{ImageBuffer, Rgba, GenericImageView};
use cairo::Context;
use crate::core::Point;

/// Represents a rectangle with position and size
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}

/// Selection type - determines how selection operations work
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionType {
    /// Replace the current selection
    New,
    /// Add to the current selection
    Add,
    /// Subtract from the current selection
    Subtract,
    /// Intersect with the current selection
    Intersect,
}

/// Selection shape - determines the initial shape of the selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionShape {
    /// Rectangular selection
    Rectangle,
    /// Elliptical selection
    Ellipse,
    /// Freehand lasso selection
    Lasso,
    /// Polygon selection with straight lines
    Polygon,
    /// Magic wand selection (by color)
    MagicWand,
}

/// Represents a selection in the image
#[derive(Clone)]
pub struct Selection {
    /// X coordinate of the selection origin
    pub x: f64,
    /// Y coordinate of the selection origin
    pub y: f64,
    /// Width of the selection
    pub width: u32,
    /// Height of the selection
    pub height: u32,
    /// Mask for the selection (8-bit grayscale where 255 = fully selected)
    pub mask: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// Type of selection operation
    pub selection_type: SelectionType,
    /// Shape of the selection
    pub shape: SelectionShape,
    /// Points for polygon or lasso selection
    pub points: Vec<Point>,
    /// Whether the selection is currently being modified
    pub is_active: bool,
}

impl Selection {
    /// Create a new empty selection
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
            mask: ImageBuffer::new(width, height),
            selection_type: SelectionType::New,
            shape: SelectionShape::Rectangle,
            points: Vec::new(),
            is_active: false,
        }
    }
    
    /// Create a rectangular selection
    pub fn rectangle(x: f64, y: f64, width: u32, height: u32, canvas_width: u32, canvas_height: u32) -> Self {
        let mut selection = Self::new(canvas_width, canvas_height);
        selection.x = x;
        selection.y = y;
        selection.width = width;
        selection.height = height;
        selection.shape = SelectionShape::Rectangle;
        
        // Create the mask for the rectangle
        for py in 0..canvas_height {
            for px in 0..canvas_width {
                let px_f64 = px as f64;
                let py_f64 = py as f64;
                let width_f64 = width as f64;
                let height_f64 = height as f64;
                
                let in_rect = (px_f64 >= x) && 
                              (px_f64 < (x + width_f64)) && 
                              (py_f64 >= y) && 
                              (py_f64 < (y + height_f64));
                
                let pixel = if in_rect {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([0, 0, 0, 0])
                };
                
                selection.mask.put_pixel(px, py, pixel);
            }
        }
        
        selection
    }
    
    /// Create an elliptical selection
    pub fn ellipse(x: f64, y: f64, width: u32, height: u32, canvas_width: u32, canvas_height: u32) -> Self {
        let mut selection = Self::new(canvas_width, canvas_height);
        selection.x = x;
        selection.y = y;
        selection.width = width;
        selection.height = height;
        selection.shape = SelectionShape::Ellipse;
        
        // Create the mask for the ellipse
        let center_x = x + width as f64 / 2.0;
        let center_y = y + height as f64 / 2.0;
        let radius_x = width as f64 / 2.0;
        let radius_y = height as f64 / 2.0;
        
        for py in 0..canvas_height {
            for px in 0..canvas_width {
                let dx = (px as f64 - center_x) / radius_x;
                let dy = (py as f64 - center_y) / radius_y;
                let distance = dx * dx + dy * dy;
                
                let pixel = if distance <= 1.0 {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([0, 0, 0, 0])
                };
                
                selection.mask.put_pixel(px, py, pixel);
            }
        }
        
        selection
    }
    
    /// Create a lasso (freehand) selection
    pub fn lasso(points: Vec<Point>, canvas_width: u32, canvas_height: u32) -> Self {
        let mut selection = Self::new(canvas_width, canvas_height);
        selection.points = points.clone();
        selection.shape = SelectionShape::Lasso;
        
        // Find the bounding box
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        
        for point in &points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
        
        selection.x = min_x;
        selection.y = min_y;
        selection.width = (max_x - min_x).ceil() as u32;
        selection.height = (max_y - min_y).ceil() as u32;
        
        // Create mask using point-in-polygon algorithm
        for py in 0..canvas_height {
            for px in 0..canvas_width {
                let point = Point::new(px as f64, py as f64);
                let pixel = if Selection::point_in_polygon(&point, &points) {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([0, 0, 0, 0])
                };
                
                selection.mask.put_pixel(px, py, pixel);
            }
        }
        
        selection
    }
    
    /// Create a polygon selection
    pub fn polygon(points: Vec<Point>, canvas_width: u32, canvas_height: u32) -> Self {
        // Polygon implementation is the same as lasso but with straight lines
        let mut selection = Self::lasso(points, canvas_width, canvas_height);
        selection.shape = SelectionShape::Polygon;
        selection
    }
    
    /// Create a magic wand selection
    pub fn magic_wand(
        x: u32, 
        y: u32, 
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>, 
        tolerance: u8
    ) -> Self {
        let width = image.width();
        let height = image.height();
        let mut selection = Self::new(width, height);
        selection.shape = SelectionShape::MagicWand;
        
        // Get target color
        let target_color = image.get_pixel(x, y);
        
        // Helper function to check if a color is within tolerance
        let is_similar = |color1: &Rgba<u8>, color2: &Rgba<u8>| -> bool {
            let diff_r = (color1[0] as i32 - color2[0] as i32).abs() as u8;
            let diff_g = (color1[1] as i32 - color2[1] as i32).abs() as u8;
            let diff_b = (color1[2] as i32 - color2[2] as i32).abs() as u8;
            let diff_a = (color1[3] as i32 - color2[3] as i32).abs() as u8;
            
            let max_diff = diff_r.max(diff_g).max(diff_b).max(diff_a);
            max_diff <= tolerance
        };
        
        // Flood fill algorithm
        let mut stack = vec![(x, y)];
        let mut visited = vec![vec![false; height as usize]; width as usize];
        
        while let Some((px, py)) = stack.pop() {
            if visited[px as usize][py as usize] {
                continue;
            }
            
            visited[px as usize][py as usize] = true;
            let current_color = image.get_pixel(px, py);
            
            if is_similar(current_color, target_color) {
                // Add to selection
                selection.mask.put_pixel(px, py, Rgba([255, 255, 255, 255]));
                
                // Add neighbors to stack
                if px > 0 && !visited[(px - 1) as usize][py as usize] {
                    stack.push((px - 1, py));
                }
                if px < width - 1 && !visited[(px + 1) as usize][py as usize] {
                    stack.push((px + 1, py));
                }
                if py > 0 && !visited[px as usize][(py - 1) as usize] {
                    stack.push((px, py - 1));
                }
                if py < height - 1 && !visited[px as usize][(py + 1) as usize] {
                    stack.push((px, py + 1));
                }
            }
        }
        
        // Find the bounding box
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        
        for y in 0..height {
            for x in 0..width {
                if selection.mask.get_pixel(x, y)[0] == 255 {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }
        
        selection.x = min_x as f64;
        selection.y = min_y as f64;
        selection.width = (max_x - min_x) as u32 + 1;
        selection.height = (max_y - min_y) as u32 + 1;
        
        selection
    }
    
    /// Combine with another selection based on the selection type
    pub fn combine(&mut self, other: &Selection) {
        match other.selection_type {
            SelectionType::New => {
                // Replace the current selection
                self.x = other.x;
                self.y = other.y;
                self.width = other.width;
                self.height = other.height;
                self.mask = other.mask.clone();
                self.shape = other.shape;
                self.points = other.points.clone();
            },
            SelectionType::Add => {
                // Add to the current selection
                for y in 0..self.mask.height() {
                    for x in 0..self.mask.width() {
                        let current = self.mask.get_pixel(x, y)[0];
                        let other_val = other.mask.get_pixel(x, y)[0];
                        
                        // Union operation (max)
                        let new_val = current.max(other_val);
                        self.mask.put_pixel(x, y, Rgba([new_val, new_val, new_val, 255]));
                    }
                }
            },
            SelectionType::Subtract => {
                // Subtract from the current selection
                for y in 0..self.mask.height() {
                    for x in 0..self.mask.width() {
                        let current = self.mask.get_pixel(x, y)[0];
                        let other_val = other.mask.get_pixel(x, y)[0];
                        
                        // Subtract operation (remove where other is selected)
                        let new_val = if other_val > 0 { 0 } else { current };
                        self.mask.put_pixel(x, y, Rgba([new_val, new_val, new_val, 255]));
                    }
                }
            },
            SelectionType::Intersect => {
                // Intersect with the current selection
                for y in 0..self.mask.height() {
                    for x in 0..self.mask.width() {
                        let current = self.mask.get_pixel(x, y)[0];
                        let other_val = other.mask.get_pixel(x, y)[0];
                        
                        // Intersection operation (min)
                        let new_val = if current > 0 && other_val > 0 { 255 } else { 0 };
                        self.mask.put_pixel(x, y, Rgba([new_val, new_val, new_val, 255]));
                    }
                }
            },
        }
        
        // Update bounding box
        self.update_bounds();
    }
    
    /// Update the selection bounds based on the mask content
    fn update_bounds(&mut self) {
        let width = self.mask.width();
        let height = self.mask.height();
        
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut has_selection = false;
        
        for y in 0..height {
            for x in 0..width {
                if self.mask.get_pixel(x, y)[0] > 0 {
                    has_selection = true;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }
        
        if has_selection {
            self.x = min_x as f64;
            self.y = min_y as f64;
            self.width = (max_x - min_x) as u32 + 1;
            self.height = (max_y - min_y) as u32 + 1;
        } else {
            // No selection, reset to empty
            self.x = 0.0;
            self.y = 0.0;
            self.width = 0;
            self.height = 0;
        }
    }
    
    /// Invert the selection
    pub fn invert(&mut self) {
        for y in 0..self.mask.height() {
            for x in 0..self.mask.width() {
                let current = self.mask.get_pixel(x, y)[0];
                let new_val = 255 - current;
                self.mask.put_pixel(x, y, Rgba([new_val, new_val, new_val, 255]));
            }
        }
        self.update_bounds();
    }
    
    /// Clear the selection
    pub fn clear(&mut self) {
        for y in 0..self.mask.height() {
            for x in 0..self.mask.width() {
                self.mask.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
        self.width = 0;
        self.height = 0;
        self.points.clear();
    }
    
    /// Feather the selection by a given radius
    pub fn feather(&mut self, radius: f64) {
        // Create a copy of the original mask
        let original = self.mask.clone();
        let width = self.mask.width();
        let height = self.mask.height();
        
        // Apply Gaussian-like blur to the mask
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0;
                let mut weight_sum = 0.0;
                
                // Kernel size based on radius
                let kernel_size = (radius as i32 * 2 + 1).max(3);
                let half_kernel = kernel_size / 2;
                
                for ky in -half_kernel..=half_kernel {
                    for kx in -half_kernel..=half_kernel {
                        let sample_x = (x as i32 + kx).clamp(0, width as i32 - 1) as u32;
                        let sample_y = (y as i32 + ky).clamp(0, height as i32 - 1) as u32;
                        
                        let dist = (kx * kx + ky * ky) as f64;
                        let weight = (-dist / (2.0 * radius * radius)).exp();
                        
                        let sample_val = original.get_pixel(sample_x, sample_y)[0] as f64 / 255.0;
                        sum += sample_val * weight;
                        weight_sum += weight;
                    }
                }
                
                let avg = (sum / weight_sum * 255.0).round().clamp(0.0, 255.0) as u8;
                self.mask.put_pixel(x, y, Rgba([avg, avg, avg, 255]));
            }
        }
    }
    
    /// Grow the selection by a given number of pixels
    pub fn grow(&mut self, amount: u32) {
        // Create a copy of the original mask
        let original = self.mask.clone();
        let width = self.mask.width();
        let height = self.mask.height();
        
        // Apply dilation operation
        for y in 0..height {
            for x in 0..width {
                // If the pixel is already selected, keep it selected
                if original.get_pixel(x, y)[0] > 0 {
                    continue;
                }
                
                // Check if any pixel within the radius is selected
                let mut should_select = false;
                
                for dy in -(amount as i32)..=(amount as i32) {
                    for dx in -(amount as i32)..=(amount as i32) {
                        // Check if we're within the radius
                        if dx * dx + dy * dy > (amount * amount) as i32 {
                            continue;
                        }
                        
                        let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as u32;
                        let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as u32;
                        
                        if original.get_pixel(nx, ny)[0] > 0 {
                            should_select = true;
                            break;
                        }
                    }
                    if should_select {
                        break;
                    }
                }
                
                if should_select {
                    self.mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
        }
        
        self.update_bounds();
    }
    
    /// Shrink the selection by a given number of pixels
    pub fn shrink(&mut self, amount: u32) {
        // Create a copy of the original mask
        let original = self.mask.clone();
        let width = self.mask.width();
        let height = self.mask.height();
        
        // Apply erosion operation
        for y in 0..height {
            for x in 0..width {
                // If the pixel is already unselected, keep it unselected
                if original.get_pixel(x, y)[0] == 0 {
                    continue;
                }
                
                // Check if any pixel within the radius is unselected
                let mut should_unselect = false;
                
                for dy in -(amount as i32)..=(amount as i32) {
                    for dx in -(amount as i32)..=(amount as i32) {
                        // Check if we're within the radius
                        if dx * dx + dy * dy > (amount * amount) as i32 {
                            continue;
                        }
                        
                        let nx = (x as i32 + dx).clamp(0, width as i32 - 1) as u32;
                        let ny = (y as i32 + dy).clamp(0, height as i32 - 1) as u32;
                        
                        if original.get_pixel(nx, ny)[0] == 0 {
                            should_unselect = true;
                            break;
                        }
                    }
                    if should_unselect {
                        break;
                    }
                }
                
                if should_unselect {
                    self.mask.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        
        self.update_bounds();
    }
    
    /// Crop the selection to the specified rectangle
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
        // Create a new mask with the cropped dimensions
        let mut new_mask = ImageBuffer::new(width, height);
        
        // Copy the relevant portion of the mask
        for py in 0..height {
            for px in 0..width {
                let src_x = px + x;
                let src_y = py + y;
                
                if src_x < self.mask.width() && src_y < self.mask.height() {
                    let pixel = self.mask.get_pixel(src_x, src_y);
                    new_mask.put_pixel(px, py, *pixel);
                } else {
                    new_mask.put_pixel(px, py, Rgba([0, 0, 0, 0]));
                }
            }
        }
        
        // Update the selection
        self.mask = new_mask;
        self.x -= x as f64;
        self.y -= y as f64;
        self.update_bounds();
    }
    
    /// Render the selection outline to a Cairo context
    pub fn render_outline(&self, context: &Context) {
        // Draw a "marching ants" style selection border
        context.save();
        
        // Set up dashed line for marching ants effect
        let dashes = [4.0, 4.0]; // Pattern of dash and gap
        context.set_dash(&dashes, 0.0);
        context.set_line_width(1.0);
        context.set_source_rgba(1.0, 1.0, 1.0, 0.8);
        
        // Draw the appropriate shape
        match self.shape {
            SelectionShape::Rectangle => {
                context.rectangle(
                    self.x, 
                    self.y, 
                    self.width as f64, 
                    self.height as f64
                );
            },
            SelectionShape::Ellipse => {
                let center_x = self.x + self.width as f64 / 2.0;
                let center_y = self.y + self.height as f64 / 2.0;
                let radius_x = self.width as f64 / 2.0;
                let radius_y = self.height as f64 / 2.0;
                
                context.save();
                context.translate(center_x, center_y);
                context.scale(radius_x, radius_y);
                context.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
                context.restore();
            },
            SelectionShape::Lasso | SelectionShape::Polygon => {
                if !self.points.is_empty() {
                    context.move_to(self.points[0].x, self.points[0].y);
                    
                    for point in &self.points[1..] {
                        context.line_to(point.x, point.y);
                    }
                    
                    // Close the path if not in active drawing
                    if !self.is_active {
                        context.close_path();
                    }
                }
            },
            SelectionShape::MagicWand => {
                // For magic wand, trace the outline of the mask
                let width = self.mask.width();
                let height = self.mask.height();
                
                // Find the edges of the selection
                let mut started = false;
                
                // This is a simple edge detection approach
                // A more sophisticated algorithm would use contour tracing
                for y in 0..height {
                    for x in 0..width {
                        let current = self.mask.get_pixel(x, y)[0] > 0;
                        
                        // Check if this pixel is at the edge of the selection
                        let is_edge = current && (
                            (x == 0 || self.mask.get_pixel(x - 1, y)[0] == 0) ||
                            (x == width - 1 || self.mask.get_pixel(x + 1, y)[0] == 0) ||
                            (y == 0 || self.mask.get_pixel(x, y - 1)[0] == 0) ||
                            (y == height - 1 || self.mask.get_pixel(x, y + 1)[0] == 0)
                        );
                        
                        if is_edge {
                            if !started {
                                context.move_to(x as f64, y as f64);
                                started = true;
                            } else {
                                context.line_to(x as f64, y as f64);
                            }
                        }
                    }
                }
            }
        }
        
        context.stroke();
        context.restore();
    }
    
    /// Check if a point is inside the selection
    pub fn contains_point(&self, point: &Point) -> bool {
        if point.x < self.x || point.x >= self.x + self.width as f64 || 
           point.y < self.y || point.y >= self.y + self.height as f64 {
            return false;
        }
        
        let px = point.x as u32;
        let py = point.y as u32;
        
        if px < self.mask.width() && py < self.mask.height() {
            return self.mask.get_pixel(px, py)[0] > 0;
        }
        
        false
    }
    
    /// Helper function: check if a point is inside a polygon using ray casting algorithm
    fn point_in_polygon(point: &Point, polygon: &[Point]) -> bool {
        if polygon.len() < 3 {
            return false;
        }
        
        let mut inside = false;
        let mut j = polygon.len() - 1;
        
        for i in 0..polygon.len() {
            let vi = &polygon[i];
            let vj = &polygon[j];
            
            if ((vi.y > point.y) != (vj.y > point.y)) && 
               (point.x < vi.x + (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y)) {
                inside = !inside;
            }
            
            j = i;
        }
        
        inside
    }
    
    /// Get the bounds of the selection as a Rect
    pub fn get_bounds(&self) -> Option<Rect> {
        if self.width == 0 || self.height == 0 {
            None
        } else {
            Some(Rect {
                x: self.x,
                y: self.y,
                width: self.width as f64,
                height: self.height as f64,
            })
        }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width as f64 &&
        y >= self.y && y <= self.y + self.height as f64
    }
} 