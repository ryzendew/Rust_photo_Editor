use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use image::{ImageBuffer, Rgba, RgbaImage};
use rust_photo::selection::{Selection, SelectionType, SelectionModifier, MaskCreationOptions};
use rust_photo::vector::{Point, Rect};

fn create_large_selection(width: u32, height: u32) -> Selection {
    let mut selection = Selection::new(width, height);
    
    // Add a rectangular selection in the center
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let rect_width = width as f32 * 0.5;
    let rect_height = height as f32 * 0.5;
    
    let rect = Rect::new(
        center_x - rect_width / 2.0,
        center_y - rect_height / 2.0,
        rect_width,
        rect_height
    );
    
    selection.add_rectangular(rect, SelectionModifier::New);
    selection
}

fn create_circular_selection(width: u32, height: u32) -> Selection {
    let mut selection = Selection::new(width, height);
    
    // Add a circular selection in the center
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = width.min(height) as f32 * 0.25;
    
    selection.add_circular(
        Point::new(center_x, center_y), 
        radius, 
        SelectionModifier::New
    );
    
    selection
}

fn bench_selection_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("SelectionCreation");
    
    // Test different image sizes
    for size in [512, 1024, 2048].iter() {
        group.bench_with_input(
            BenchmarkId::new("Rectangular", format!("size={}", size)), 
            size, 
            |b, size| {
                b.iter(|| {
                    let selection = create_large_selection(*size, *size);
                    black_box(selection)
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("Circular", format!("size={}", size)), 
            size, 
            |b, size| {
                b.iter(|| {
                    let selection = create_circular_selection(*size, *size);
                    black_box(selection)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_selection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("SelectionOperations");
    
    let size = 1024;
    let rect_selection = create_large_selection(size, size);
    let circle_selection = create_circular_selection(size, size);
    
    group.bench_function("Intersection", |b| {
        b.iter(|| {
            let result = rect_selection.clone().intersect(&circle_selection);
            black_box(result)
        })
    });
    
    group.bench_function("Union", |b| {
        b.iter(|| {
            let result = rect_selection.clone().union(&circle_selection);
            black_box(result)
        })
    });
    
    group.bench_function("Subtract", |b| {
        b.iter(|| {
            let result = rect_selection.clone().subtract(&circle_selection);
            black_box(result)
        })
    });
    
    group.bench_function("Feather", |b| {
        b.iter(|| {
            let mut selection = rect_selection.clone();
            selection.feather(10.0);
            black_box(selection)
        })
    });
    
    group.bench_function("Grow", |b| {
        b.iter(|| {
            let mut selection = rect_selection.clone();
            selection.grow(10);
            black_box(selection)
        })
    });
    
    group.bench_function("Shrink", |b| {
        b.iter(|| {
            let mut selection = rect_selection.clone();
            selection.shrink(10);
            black_box(selection)
        })
    });
    
    group.bench_function("Border", |b| {
        b.iter(|| {
            let mut selection = rect_selection.clone();
            selection.border(10);
            black_box(selection)
        })
    });
    
    group.finish();
}

fn bench_mask_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("MaskCreation");
    
    let size = 1024;
    let rect_selection = create_large_selection(size, size);
    let circle_selection = create_circular_selection(size, size);
    let combined_selection = rect_selection.clone().union(&circle_selection);
    
    // Create test image
    let mut test_image = RgbaImage::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let r = ((x as f32 / size as f32) * 255.0) as u8;
            let g = ((y as f32 / size as f32) * 255.0) as u8;
            let b = (((x+y) as f32 / (size*2) as f32) * 255.0) as u8;
            test_image.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    
    group.bench_function("Create Basic Mask", |b| {
        b.iter(|| {
            let mask = combined_selection.create_mask(MaskCreationOptions::default());
            black_box(mask)
        })
    });
    
    group.bench_function("Create Feathered Mask", |b| {
        b.iter(|| {
            let options = MaskCreationOptions {
                feather: 10.0,
                ..Default::default()
            };
            let mask = combined_selection.create_mask(options);
            black_box(mask)
        })
    });
    
    group.bench_function("Apply Mask to Image", |b| {
        b.iter(|| {
            let mask = combined_selection.create_mask(MaskCreationOptions::default());
            let result = combined_selection.apply_to_image(&test_image);
            black_box(result)
        })
    });
    
    group.finish();
}

fn bench_selection_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("SelectionTransform");
    
    let size = 1024;
    let selection = create_large_selection(size, size);
    
    group.bench_function("Scale", |b| {
        b.iter(|| {
            let mut scaled = selection.clone();
            scaled.scale(1.5, 1.5);
            black_box(scaled)
        })
    });
    
    group.bench_function("Rotate", |b| {
        b.iter(|| {
            let mut rotated = selection.clone();
            rotated.rotate(45.0);
            black_box(rotated)
        })
    });
    
    group.bench_function("Translate", |b| {
        b.iter(|| {
            let mut translated = selection.clone();
            translated.translate(100.0, 100.0);
            black_box(translated)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_selection_creation,
    bench_selection_operations,
    bench_mask_creation,
    bench_selection_transform
);
criterion_main!(benches); 