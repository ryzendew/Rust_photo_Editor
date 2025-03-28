use std::fmt;

// Trait for commands that can be undone/redone
pub trait HistoryCommand: fmt::Debug {
    fn execute(&mut self) -> bool;
    fn undo(&mut self) -> bool;
    fn redo(&mut self) -> bool {
        self.execute()
    }
    fn get_name(&self) -> String;
}

// A single state in the history stack
#[derive(Debug)]
pub struct HistoryState {
    command: Box<dyn HistoryCommand>,
    document_id: String,
}

impl HistoryState {
    pub fn new(command: Box<dyn HistoryCommand>, document_id: String) -> Self {
        Self {
            command,
            document_id,
        }
    }
    
    pub fn undo(&mut self) -> bool {
        self.command.undo()
    }
    
    pub fn redo(&mut self) -> bool {
        self.command.redo()
    }
    
    pub fn get_name(&self) -> String {
        self.command.get_name()
    }
    
    pub fn get_document_id(&self) -> &str {
        &self.document_id
    }
}

// Manages a stack of history states
#[derive(Debug)]
pub struct HistoryManager {
    undo_stack: Vec<HistoryState>,
    redo_stack: Vec<HistoryState>,
    max_undo_levels: usize,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: 100,
        }
    }
    
    pub fn with_max_levels(max_levels: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: max_levels,
        }
    }
    
    pub fn add_command(&mut self, mut command: Box<dyn HistoryCommand>, document_id: String) -> bool {
        // Execute the command
        let success = command.execute();
        
        if success {
            // Add to undo stack
            self.undo_stack.push(HistoryState::new(command, document_id));
            
            // Clear redo stack
            self.redo_stack.clear();
            
            // Trim undo stack if needed
            if self.undo_stack.len() > self.max_undo_levels {
                self.undo_stack.remove(0);
            }
        }
        
        success
    }
    
    pub fn undo(&mut self) -> bool {
        if let Some(mut state) = self.undo_stack.pop() {
            let success = state.undo();
            
            if success {
                self.redo_stack.push(state);
                return true;
            } else {
                // Put it back if undo failed
                self.undo_stack.push(state);
            }
        }
        
        false
    }
    
    pub fn redo(&mut self) -> bool {
        if let Some(mut state) = self.redo_stack.pop() {
            let success = state.redo();
            
            if success {
                self.undo_stack.push(state);
                return true;
            } else {
                // Put it back if redo failed
                self.redo_stack.push(state);
            }
        }
        
        false
    }
    
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    pub fn get_undo_name(&self) -> Option<String> {
        self.undo_stack.last().map(|state| state.get_name())
    }
    
    pub fn get_redo_name(&self) -> Option<String> {
        self.redo_stack.last().map(|state| state.get_name())
    }
    
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
    
    pub fn clear_document_history(&mut self, document_id: &str) {
        self.undo_stack.retain(|state| state.get_document_id() != document_id);
        self.redo_stack.retain(|state| state.get_document_id() != document_id);
    }
    
    pub fn set_max_undo_levels(&mut self, levels: usize) {
        self.max_undo_levels = levels;
        
        // Trim undo stack if needed
        while self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.remove(0);
        }
    }
}

// Example command implementations for common operations

// Layer visibility change command
#[derive(Debug)]
pub struct LayerVisibilityCommand {
    layer_id: String,
    old_visibility: bool,
    new_visibility: bool,
    applied: bool,
}

impl LayerVisibilityCommand {
    pub fn new(layer_id: String, old_visibility: bool, new_visibility: bool) -> Self {
        Self {
            layer_id,
            old_visibility,
            new_visibility,
            applied: false,
        }
    }
}

impl HistoryCommand for LayerVisibilityCommand {
    fn execute(&mut self) -> bool {
        // This would normally modify the document layer's visibility
        println!("Setting layer {} visibility to {}", self.layer_id, self.new_visibility);
        self.applied = true;
        true
    }
    
    fn undo(&mut self) -> bool {
        if !self.applied {
            return false;
        }
        
        // Revert back to old visibility
        println!("Reverting layer {} visibility to {}", self.layer_id, self.old_visibility);
        true
    }
    
    fn get_name(&self) -> String {
        format!("Change Layer Visibility")
    }
}

// Add more command implementations here... 