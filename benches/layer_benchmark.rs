use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use image::{ImageBuffer, Rgba};
use rust_photo::core::layers::{Layer, LayerManager, LayerBlendMode};
use std::sync::Arc;

fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = ImageBuffer::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            image.put_pixel(x, y, color);
        }
    }
    
    image
}

fn create_gradient_image(width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = ImageBuffer::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = (((x+y) as f32 / (width+height) as f32) * 255.0) as u8;
            image.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    
    image
}

fn bench_layer_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("LayerRendering");
    
    // Test different image sizes
    for size in [256, 512, 1024].iter() {
        let mut layer_manager = LayerManager::new();
        
        // Add several layers with different blend modes
        let base_image = create_gradient_image(*size, *size);
        let base_layer = Layer::from_image("Base layer".to_string(), base_image);
        let base_layer_id = base_layer.id();
        layer_manager.add_layer(base_layer);
        
        let overlay_image = create_test_image(*size, *size, Rgba([255, 0, 0, 128]));
        let mut overlay_layer = Layer::from_image("Overlay".to_string(), overlay_image);
        overlay_layer.set_blend_mode(LayerBlendMode::Overlay);
        let overlay_id = overlay_layer.id();
        layer_manager.add_layer(overlay_layer);
        
        let multiply_image = create_test_image(*size, *size, Rgba([0, 255, 0, 128]));
        let mut multiply_layer = Layer::from_image("Multiply".to_string(), multiply_image);
        multiply_layer.set_blend_mode(LayerBlendMode::Multiply);
        let multiply_id = multiply_layer.id();
        layer_manager.add_layer(multiply_layer);
        
        let screen_image = create_test_image(*size, *size, Rgba([0, 0, 255, 128]));
        let mut screen_layer = Layer::from_image("Screen".to_string(), screen_image);
        screen_layer.set_blend_mode(LayerBlendMode::Screen);
        let screen_id = screen_layer.id();
        layer_manager.add_layer(screen_layer);
        
        group.bench_with_input(
            BenchmarkId::new("Single Layer Render", format!("size={}", size)), 
            size, 
            |b, _| {
                b.iter(|| {
                    let result = layer_manager.render_layer(black_box(base_layer_id));
                    black_box(result)
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("Full Composite", format!("size={}", size)), 
            size, 
            |b, _| {
                b.iter(|| {
                    let result = layer_manager.render();
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_blend_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("BlendModes");
    
    let size = 512;
    let base_image = create_gradient_image(size, size);
    let overlay_image = create_test_image(size, size, Rgba([255, 0, 0, 128]));
    
    // Test different blend modes
    let blend_modes = vec![
        LayerBlendMode::Normal,
        LayerBlendMode::Multiply,
        LayerBlendMode::Screen,
        LayerBlendMode::Overlay,
        LayerBlendMode::Darken,
        LayerBlendMode::Lighten,
        LayerBlendMode::ColorDodge,
        LayerBlendMode::ColorBurn,
        LayerBlendMode::HardLight,
        LayerBlendMode::SoftLight,
        LayerBlendMode::Difference,
        LayerBlendMode::Exclusion,
    ];
    
    for blend_mode in blend_modes {
        let mut layer_manager = LayerManager::new();
        
        let base_layer = Layer::from_image("Base layer".to_string(), base_image.clone());
        let base_id = base_layer.id();
        layer_manager.add_layer(base_layer);
        
        let mut blend_layer = Layer::from_image("Blend layer".to_string(), overlay_image.clone());
        blend_layer.set_blend_mode(blend_mode.clone());
        layer_manager.add_layer(blend_layer);
        
        group.bench_with_input(
            BenchmarkId::new(format!("{:?}", blend_mode), "size=512"), 
            &blend_mode, 
            |b, _| {
                b.iter(|| {
                    let result = layer_manager.render();
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_layer_opacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("LayerOpacity");
    
    let size = 512;
    let base_image = create_gradient_image(size, size);
    let overlay_image = create_test_image(size, size, Rgba([255, 0, 0, 255]));
    
    for opacity in [25, 50, 75, 100].iter() {
        let mut layer_manager = LayerManager::new();
        
        let base_layer = Layer::from_image("Base layer".to_string(), base_image.clone());
        let base_id = base_layer.id();
        layer_manager.add_layer(base_layer);
        
        let mut blend_layer = Layer::from_image("Opacity layer".to_string(), overlay_image.clone());
        blend_layer.set_opacity(*opacity as f32 / 100.0);
        layer_manager.add_layer(blend_layer);
        
        group.bench_with_input(
            BenchmarkId::new(format!("Opacity={}", opacity), "size=512"), 
            opacity, 
            |b, _| {
                b.iter(|| {
                    let result = layer_manager.render();
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_layer_rendering,
    bench_blend_modes,
    bench_layer_opacity
);
criterion_main!(benches); 