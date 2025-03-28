use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_photo::vector::{
    Point, Rect, Transform, VectorObject, VectorShape, 
    VectorDocument, VectorLayer
};
use std::f64::consts::PI;

fn bench_vector_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("VectorTransforms");
    
    // Create test data
    let points = vec![
        Point::new(10.0, 10.0),
        Point::new(50.0, 10.0),
        Point::new(50.0, 50.0),
        Point::new(10.0, 50.0),
    ];
    
    // Test translation
    group.bench_function("Translation", |b| {
        b.iter(|| {
            let transform = Transform::translation(100.0, 100.0);
            let result: Vec<Point> = points.iter()
                .map(|p| transform.apply_to_point(black_box(p)))
                .collect();
            black_box(result)
        })
    });
    
    // Test scaling
    group.bench_function("Scaling", |b| {
        b.iter(|| {
            let transform = Transform::scale(2.0, 2.0);
            let result: Vec<Point> = points.iter()
                .map(|p| transform.apply_to_point(black_box(p)))
                .collect();
            black_box(result)
        })
    });
    
    // Test rotation
    group.bench_function("Rotation", |b| {
        b.iter(|| {
            let transform = Transform::rotation(45.0);
            let result: Vec<Point> = points.iter()
                .map(|p| transform.apply_to_point(black_box(p)))
                .collect();
            black_box(result)
        })
    });
    
    // Test compound transformations
    group.bench_function("CompoundTransform", |b| {
        b.iter(|| {
            let t1 = Transform::translation(100.0, 100.0);
            let t2 = Transform::scale(2.0, 2.0);
            let t3 = Transform::rotation(45.0);
            
            let transform = t1.multiply(&t2).multiply(&t3);
            let result: Vec<Point> = points.iter()
                .map(|p| transform.apply_to_point(black_box(p)))
                .collect();
            black_box(result)
        })
    });
    
    // Test transform inversion
    group.bench_function("TransformInversion", |b| {
        b.iter(|| {
            let transform = Transform::scale(2.0, 3.0).multiply(
                &Transform::rotation(30.0)
            ).multiply(
                &Transform::translation(100.0, 50.0)
            );
            let inverse = transform.invert();
            black_box(inverse)
        })
    });
    
    group.finish();
}

fn bench_vector_shapes(c: &mut Criterion) {
    let mut group = c.benchmark_group("VectorShapes");
    
    // Test rectangle creation
    group.bench_function("Rectangle", |b| {
        b.iter(|| {
            let rect = VectorShape::rectangle(10.0, 10.0, 100.0, 100.0);
            black_box(rect)
        })
    });
    
    // Test circle creation
    group.bench_function("Circle", |b| {
        b.iter(|| {
            let circle = VectorShape::circle(50.0, 50.0, 40.0);
            black_box(circle)
        })
    });
    
    // Test polygon creation with many points
    group.bench_function("Polygon100", |b| {
        b.iter(|| {
            let mut points = Vec::with_capacity(100);
            let center_x = 100.0;
            let center_y = 100.0;
            let radius = 50.0;
            
            for i in 0..100 {
                let angle = (i as f64 / 100.0) * 2.0 * PI;
                let x = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();
                points.push((x, y));
            }
            
            let polygon = VectorShape::polygon(black_box(points));
            black_box(polygon)
        })
    });
    
    // Test converting vector shapes to paths
    group.bench_function("ShapeToPath", |b| {
        b.iter(|| {
            let rect = VectorShape::rectangle(10.0, 10.0, 100.0, 100.0);
            let circle = VectorShape::circle(50.0, 50.0, 40.0);
            
            let rect_path = rect.to_path();
            let circle_path = circle.to_path();
            
            black_box((rect_path, circle_path))
        })
    });
    
    group.finish();
}

fn bench_vector_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("VectorDocument");
    
    // Test document with different numbers of layers
    for num_layers in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("DocumentCreation", format!("layers={}", num_layers)), 
            num_layers, 
            |b, &num_layers| {
                b.iter(|| {
                    let mut doc = VectorDocument::new(1000, 1000);
                    
                    // Add layers with several shapes each
                    for i in 0..num_layers {
                        let layer_name = format!("Layer {}", i);
                        let layer_idx = doc.add_layer(&layer_name);
                        
                        if let Some(layer) = doc.get_active_layer_mut() {
                            // Add rectangle
                            let rect = VectorShape::rectangle(
                                10.0 + i as f64 * 5.0, 
                                10.0 + i as f64 * 5.0, 
                                100.0, 
                                100.0
                            );
                            layer.add_shape(rect);
                            
                            // Add circle
                            let circle = VectorShape::circle(
                                150.0 + i as f64 * 5.0, 
                                150.0 + i as f64 * 5.0, 
                                40.0
                            );
                            layer.add_shape(circle);
                        }
                    }
                    
                    black_box(doc)
                })
            }
        );
    }
    
    // Create a document with many layers for the remaining tests
    let mut doc = VectorDocument::new(1000, 1000);
    for i in 0..50 {
        let layer_name = format!("Layer {}", i);
        let layer_idx = doc.add_layer(&layer_name);
        
        if let Some(layer) = doc.get_active_layer_mut() {
            // Add rectangle
            let rect = VectorShape::rectangle(
                10.0 + i as f64 * 5.0, 
                10.0 + i as f64 * 5.0, 
                100.0, 
                100.0
            );
            layer.add_shape(rect);
            
            // Add circle
            let circle = VectorShape::circle(
                150.0 + i as f64 * 5.0, 
                150.0 + i as f64 * 5.0, 
                40.0
            );
            layer.add_shape(circle);
        }
    }
    
    // Test layer manipulation
    group.bench_function("LayerManipulation", |b| {
        b.iter(|| {
            let mut test_doc = black_box(doc.clone());
            
            // Test layer visibility changes
            for i in 0..test_doc.get_layers().len() {
                if i % 2 == 0 {
                    if let Some(layer) = test_doc.get_layers_mut().get_mut(i) {
                        layer.set_visible(false);
                    }
                }
            }
            
            // Test changing active layer
            for i in 0..test_doc.get_layers().len() {
                if i % 10 == 0 {
                    test_doc.set_active_layer(i);
                }
            }
            
            black_box(test_doc)
        })
    });
    
    // Test adding shapes to document
    group.bench_function("AddShapes", |b| {
        b.iter(|| {
            let mut test_doc = black_box(doc.clone());
            test_doc.set_active_layer(0);
            
            for i in 0..100 {
                let shape = if i % 2 == 0 {
                    VectorShape::rectangle(
                        200.0 + i as f64, 
                        200.0 + i as f64, 
                        50.0, 
                        50.0
                    )
                } else {
                    VectorShape::circle(
                        300.0 + i as f64, 
                        300.0 + i as f64, 
                        25.0
                    )
                };
                
                test_doc.add_shape(shape);
            }
            
            black_box(test_doc)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_vector_transforms,
    bench_vector_shapes,
    bench_vector_document
);
criterion_main!(benches); 