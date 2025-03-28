use std::collections::HashMap;
use uuid::Uuid;
use cairo::Context;
use crate::vector::{Point, Rect, VectorObject, Transform, SelectionState};
use crate::vector::shape::VectorShape;
use crate::vector::path::Path;
use crate::vector::text::TextShape;

/// A layer containing vector objects
#[derive(Debug, Clone)]
pub struct VectorLayer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub objects: Vec<Box<dyn VectorObject>>,
    pub sublayers: Vec<VectorLayer>,
}

impl VectorLayer {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            visible: true,
            locked: false,
            opacity: 1.0,
            objects: Vec::new(),
            sublayers: Vec::new(),
        }
    }
    
    pub fn add_object(&mut self, object: Box<dyn VectorObject>) {
        self.objects.push(object);
    }
    
    pub fn add_shape(&mut self, shape: VectorShape) {
        self.add_object(Box::new(shape));
    }
    
    pub fn add_text(&mut self, text: TextShape) {
        self.add_object(Box::new(text));
    }
    
    pub fn add_sublayer(&mut self, layer: VectorLayer) {
        self.sublayers.push(layer);
    }
    
    pub fn remove_object(&mut self, index: usize) -> Option<Box<dyn VectorObject>> {
        if index < self.objects.len() {
            Some(self.objects.remove(index))
        } else {
            None
        }
    }
    
    pub fn remove_sublayer(&mut self, index: usize) -> Option<VectorLayer> {
        if index < self.sublayers.len() {
            Some(self.sublayers.remove(index))
        } else {
            None
        }
    }
    
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
    
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.max(0.0).min(1.0);
    }
    
    pub fn get_object(&self, index: usize) -> Option<&Box<dyn VectorObject>> {
        self.objects.get(index)
    }
    
    pub fn get_object_mut(&mut self, index: usize) -> Option<&mut Box<dyn VectorObject>> {
        self.objects.get_mut(index)
    }
    
    pub fn get_sublayer(&self, index: usize) -> Option<&VectorLayer> {
        self.sublayers.get(index)
    }
    
    pub fn get_sublayer_mut(&mut self, index: usize) -> Option<&mut VectorLayer> {
        self.sublayers.get_mut(index)
    }
    
    pub fn find_object_by_id(&self, id: &str) -> Option<&Box<dyn VectorObject>> {
        // TODO: Implement a more efficient lookup using a HashMap
        self.objects.iter().find(|obj| {
            // This is a bit hacky since we don't have a common id field in VectorObject
            if let Some(shape) = obj.as_any().downcast_ref::<VectorShape>() {
                shape.id == id
            } else if let Some(text) = obj.as_any().downcast_ref::<TextShape>() {
                text.id == id
            } else {
                false
            }
        })
    }
    
    pub fn find_sublayer_by_id(&self, id: &str) -> Option<&VectorLayer> {
        if self.id == id {
            return Some(self);
        }
        
        for sublayer in &self.sublayers {
            if let Some(layer) = sublayer.find_sublayer_by_id(id) {
                return Some(layer);
            }
        }
        
        None
    }
    
    pub fn find_sublayer_by_id_mut(&mut self, id: &str) -> Option<&mut VectorLayer> {
        if self.id == id {
            return Some(self);
        }
        
        for sublayer in &mut self.sublayers {
            if let Some(layer) = sublayer.find_sublayer_by_id_mut(id) {
                return Some(layer);
            }
        }
        
        None
    }
    
    pub fn draw(&self, context: &Context) {
        if !self.visible {
            return;
        }
        
        context.save();
        
        // Apply layer opacity
        if self.opacity < 1.0 {
            // To properly apply opacity to a layer, we should render to an offscreen surface
            // and then composite it with the given opacity, but for simplicity, we'll just
            // adjust the alpha of all objects
            context.push_group();
        }
        
        // Draw all objects in this layer
        for object in &self.objects {
            object.draw(context);
        }
        
        // Draw all sublayers
        for sublayer in &self.sublayers {
            sublayer.draw(context);
        }
        
        // Apply layer opacity
        if self.opacity < 1.0 {
            context.pop_group_to_source();
            context.paint_with_alpha(self.opacity);
        }
        
        context.restore();
    }
}

/// A complete vector document
#[derive(Debug, Clone)]
pub struct VectorDocument {
    pub id: String,
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub layers: Vec<VectorLayer>,
    pub selected_objects: Vec<String>, // IDs of selected objects
    pub selected_layer: Option<String>, // ID of selected layer
    pub zoom: f64,
    pub view_offset: Point,
}

impl VectorDocument {
    pub fn new(name: String, width: f64, height: f64) -> Self {
        let mut doc = Self {
            id: Uuid::new_v4().to_string(),
            name,
            width,
            height,
            layers: Vec::new(),
            selected_objects: Vec::new(),
            selected_layer: None,
            zoom: 1.0,
            view_offset: Point::new(0.0, 0.0),
        };
        
        // Create a default layer
        let default_layer = VectorLayer::new("Layer 1".to_string());
        doc.layers.push(default_layer);
        doc.selected_layer = Some(doc.layers[0].id.clone());
        
        doc
    }
    
    pub fn add_layer(&mut self, name: String) -> String {
        let layer = VectorLayer::new(name);
        let id = layer.id.clone();
        self.layers.push(layer);
        id
    }
    
    pub fn remove_layer(&mut self, index: usize) -> Option<VectorLayer> {
        if index < self.layers.len() && self.layers.len() > 1 {
            let layer = self.layers.remove(index);
            
            // If we removed the selected layer, select the first layer
            if let Some(selected_id) = &self.selected_layer {
                if *selected_id == layer.id {
                    self.selected_layer = Some(self.layers[0].id.clone());
                }
            }
            
            Some(layer)
        } else {
            None
        }
    }
    
    pub fn get_active_layer(&self) -> Option<&VectorLayer> {
        match &self.selected_layer {
            Some(id) => {
                // First look at top-level layers
                for layer in &self.layers {
                    if layer.id == *id {
                        return Some(layer);
                    }
                    
                    // Then check sublayers
                    if let Some(sublayer) = layer.find_sublayer_by_id(id) {
                        return Some(sublayer);
                    }
                }
                None
            },
            None => None,
        }
    }
    
    pub fn get_active_layer_mut(&mut self) -> Option<&mut VectorLayer> {
        match &self.selected_layer {
            Some(id) => {
                let id_clone = id.clone();
                
                // First look at top-level layers
                for layer in &mut self.layers {
                    if layer.id == id_clone {
                        return Some(layer);
                    }
                    
                    // Then check sublayers
                    if let Some(sublayer) = layer.find_sublayer_by_id_mut(&id_clone) {
                        return Some(sublayer);
                    }
                }
                None
            },
            None => None,
        }
    }
    
    pub fn select_layer(&mut self, id: &str) -> bool {
        // Check if layer exists
        for layer in &self.layers {
            if layer.id == id {
                self.selected_layer = Some(id.to_string());
                return true;
            }
            
            if layer.find_sublayer_by_id(id).is_some() {
                self.selected_layer = Some(id.to_string());
                return true;
            }
        }
        
        false
    }
    
    pub fn select_object(&mut self, id: &str, add_to_selection: bool) -> bool {
        // Clear existing selection if not adding to it
        if !add_to_selection {
            self.clear_selection();
        }
        
        // Check if object exists in any layer
        for layer in &self.layers {
            if layer.find_object_by_id(id).is_some() {
                if !self.selected_objects.contains(&id.to_string()) {
                    self.selected_objects.push(id.to_string());
                }
                return true;
            }
        }
        
        false
    }
    
    pub fn deselect_object(&mut self, id: &str) -> bool {
        let original_len = self.selected_objects.len();
        self.selected_objects.retain(|obj_id| obj_id != id);
        original_len != self.selected_objects.len()
    }
    
    pub fn clear_selection(&mut self) {
        self.selected_objects.clear();
    }
    
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.max(0.1).min(16.0);
    }
    
    pub fn set_view_offset(&mut self, offset: Point) {
        self.view_offset = offset;
    }
    
    pub fn get_object_by_id(&self, id: &str) -> Option<&Box<dyn VectorObject>> {
        for layer in &self.layers {
            if let Some(obj) = layer.find_object_by_id(id) {
                return Some(obj);
            }
        }
        None
    }
    
    pub fn draw(&self, context: &Context) {
        context.save();
        
        // Apply view transformation
        context.translate(-self.view_offset.x, -self.view_offset.y);
        context.scale(self.zoom, self.zoom);
        
        // Draw document background
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.rectangle(0.0, 0.0, self.width, self.height);
        context.fill();
        
        // Draw boundary
        context.set_source_rgb(0.8, 0.8, 0.8);
        context.set_line_width(1.0 / self.zoom);
        context.rectangle(0.0, 0.0, self.width, self.height);
        context.stroke();
        
        // Draw all layers
        for layer in &self.layers {
            layer.draw(context);
        }
        
        context.restore();
    }
    
    pub fn add_shape_to_active_layer(&mut self, shape: VectorShape) -> bool {
        if let Some(layer) = self.get_active_layer_mut() {
            layer.add_shape(shape);
            true
        } else {
            false
        }
    }
    
    pub fn add_text_to_active_layer(&mut self, text: TextShape) -> bool {
        if let Some(layer) = self.get_active_layer_mut() {
            layer.add_text(text);
            true
        } else {
            false
        }
    }
    
    pub fn export_as_svg(&self, path: &str) -> Result<(), String> {
        // This would implement SVG export
        // For now, return a placeholder
        Ok(())
    }
} 