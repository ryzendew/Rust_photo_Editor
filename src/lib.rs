// Rust Photo - Library
// Exports all modules for the application

// Import modules
pub mod core;
pub mod filters;
pub mod tools;
pub mod ui;
pub mod vector;
pub mod raw;

// Re-export commonly used types
pub use core::canvas::Canvas;
pub use core::layer::{Layer, LayerManager};
pub use core::canvas::Tool;
pub use vector::VectorShape;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Rust Photo";
pub const APP_ID: &str = "com.example.rust_photo";

// GPU acceleration detection
pub fn has_gpu_support() -> bool {
    has_vulkan_support() || has_wgpu_support()
}

/// Check if Vulkan support is available
pub fn has_vulkan_support() -> bool {
    #[cfg(feature = "gpu-vulkan")]
    {
        // Implementation would check for Vulkan
        true
    }
    #[cfg(not(feature = "gpu-vulkan"))]
    {
        false
    }
}

/// Check if WebGPU support is available
pub fn has_wgpu_support() -> bool {
    #[cfg(feature = "gpu-wgpu")]
    {
        // Implementation would check for WebGPU
        true
    }
    #[cfg(not(feature = "gpu-wgpu"))]
    {
        false
    }
}

/// Initialize GPU support
pub fn init_gpu() -> Result<(), String> {
    if has_gpu_support() {
        // Try to initialize available GPU backends in order of preference
        if has_vulkan_support() {
            return init_vulkan();
        } else if has_wgpu_support() {
            return init_wgpu();
        }
    }
    
    Ok(())
}

fn init_vulkan() -> Result<(), String> {
    #[cfg(feature = "gpu-vulkan")]
    {
        // Vulkan initialization would go here
    }
    
    Ok(())
}

fn init_wgpu() -> Result<(), String> {
    #[cfg(feature = "gpu-wgpu")]
    {
        // WebGPU initialization would go here
    }
    
    Ok(())
} 