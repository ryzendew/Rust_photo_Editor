use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use image::{ImageBuffer, Rgba};
use rust_photo::filters::{
    BoxBlur, GaussianBlur, MotionBlur, RadialBlur, UnsharpMask, Sharpen, Filter,
    apply_filter, apply_filter_parallel
};

fn create_test_image(width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = ImageBuffer::new(width, height);
    
    // Fill with a test pattern
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

fn bench_gaussian_blur(c: &mut Criterion) {
    let mut group = c.benchmark_group("GaussianBlur");
    
    // Test different image sizes
    for size in [256, 512, 1024].iter() {
        let image = create_test_image(*size, *size);
        
        // Test different radius values
        for radius in [1.0, 3.0, 5.0].iter() {
            let filter = GaussianBlur::new(*radius);
            
            group.bench_with_input(
                BenchmarkId::new(format!("size={}", size), format!("radius={}", radius)), 
                &image, 
                |b, img| {
                    b.iter(|| {
                        let result = filter.apply(black_box(img));
                        black_box(result)
                    })
                }
            );
        }
    }
    
    group.finish();
}

fn bench_box_blur(c: &mut Criterion) {
    let mut group = c.benchmark_group("BoxBlur");
    
    // Test different image sizes
    for size in [256, 512, 1024].iter() {
        let image = create_test_image(*size, *size);
        
        // Test different radius values
        for radius in [1, 3, 5].iter() {
            let filter = BoxBlur::new(*radius);
            
            group.bench_with_input(
                BenchmarkId::new(format!("size={}", size), format!("radius={}", radius)), 
                &image, 
                |b, img| {
                    b.iter(|| {
                        let result = filter.apply(black_box(img));
                        black_box(result)
                    })
                }
            );
        }
    }
    
    group.finish();
}

fn bench_motion_blur(c: &mut Criterion) {
    let mut group = c.benchmark_group("MotionBlur");
    
    let size = 512;
    let image = create_test_image(size, size);
    
    // Test different angles and distances
    for angle in [0.0, 45.0, 90.0].iter() {
        for distance in [5, 10, 20].iter() {
            let filter = MotionBlur::new(*angle, *distance);
            
            group.bench_with_input(
                BenchmarkId::new(format!("angle={}", angle), format!("distance={}", distance)), 
                &image, 
                |b, img| {
                    b.iter(|| {
                        let result = filter.apply(black_box(img));
                        black_box(result)
                    })
                }
            );
        }
    }
    
    group.finish();
}

fn bench_unsharp_mask(c: &mut Criterion) {
    let mut group = c.benchmark_group("UnsharpMask");
    
    let size = 512;
    let image = create_test_image(size, size);
    
    // Test different radius and amount values
    for radius in [1.0, 3.0, 5.0].iter() {
        for amount in [0.5, 1.0, 2.0].iter() {
            let filter = UnsharpMask::new(*radius, *amount, 0);
            
            group.bench_with_input(
                BenchmarkId::new(format!("radius={}", radius), format!("amount={}", amount)), 
                &image, 
                |b, img| {
                    b.iter(|| {
                        let result = filter.apply(black_box(img));
                        black_box(result)
                    })
                }
            );
        }
    }
    
    group.finish();
}

fn bench_parallel_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("ParallelVsSequential");
    
    let size = 1024;
    let image = create_test_image(size, size);
    let radius = 3.0;
    
    // Sequential application
    group.bench_function("Sequential", |b| {
        b.iter(|| {
            let filter = GaussianBlur::new(radius);
            let result = filter.apply(black_box(&image));
            black_box(result)
        })
    });
    
    // Parallel application with different thread counts
    for threads in [2, 4, 8].iter() {
        group.bench_function(format!("Parallel-{}", threads), |b| {
            b.iter(|| {
                let result = apply_filter_parallel(
                    black_box(&image),
                    || Box::new(GaussianBlur::new(radius)),
                    *threads
                );
                black_box(result)
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_gaussian_blur,
    bench_box_blur,
    bench_motion_blur,
    bench_unsharp_mask,
    bench_parallel_vs_sequential
);
criterion_main!(benches); 