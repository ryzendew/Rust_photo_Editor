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
pub use core::layers::{Layer, LayerManager};
pub use core::canvas::Tool;
pub use vector::VectorShape;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Rust Photo";
pub const APP_ID: &str = "com.example.rust_photo";

// GPU acceleration detection
pub fn has_gpu_support() -> bool {
    has_cuda_support() || has_rocm_support() || has_opencl_support() || has_wgpu_support()
}

/// Check if CUDA support is available
pub fn has_cuda_support() -> bool {
    #[cfg(feature = "gpu-cuda")]
    {
        // Implementation would check for CUDA
        true
    }
    #[cfg(not(feature = "gpu-cuda"))]
    {
        false
    }
}

/// Check if ROCm support is available
pub fn has_rocm_support() -> bool {
    #[cfg(feature = "gpu-rocm")]
    {
        // Implementation would check for ROCm
        true
    }
    #[cfg(not(feature = "gpu-rocm"))]
    {
        false
    }
}

/// Check if OpenCL support is available
pub fn has_opencl_support() -> bool {
    #[cfg(feature = "gpu-opencl")]
    {
        // Implementation would check for OpenCL
        true
    }
    #[cfg(not(feature = "gpu-opencl"))]
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
        if has_cuda_support() {
            return init_cuda();
        } else if has_rocm_support() {
            return init_rocm();
        } else if has_wgpu_support() {
            return init_wgpu();
        } else if has_opencl_support() {
            return init_opencl();
        }
    }
    
    Ok(())
}

fn init_cuda() -> Result<(), String> {
    #[cfg(feature = "gpu-cuda")]
    {
        // CUDA initialization would go here
    }
    
    Ok(())
}

fn init_rocm() -> Result<(), String> {
    #[cfg(feature = "gpu-rocm")]
    {
        // ROCm initialization would go here
    }
    
    Ok(())
}

fn init_opencl() -> Result<(), String> {
    #[cfg(feature = "gpu-opencl")]
    {
        // OpenCL initialization would go here
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