use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use image::{DynamicImage, ImageBuffer, Rgba};
use rust_photo::raw::{
    RawProcessor, RawImage, RawProcessingParams, WhiteBalance, 
    DemosaicAlgorithm, NoiseReductionParams, DetailEnhancementParams
};
use std::path::PathBuf;

fn bench_raw_demosaic(c: &mut Criterion) {
    let mut group = c.benchmark_group("RAWDemosaic");
    
    // Test different demosaic algorithms
    let test_files = [
        "test_data/raw/test_image_small.raw",
        "test_data/raw/test_image_medium.raw",
    ];
    
    let algorithms = [
        DemosaicAlgorithm::Bilinear,
        DemosaicAlgorithm::VNG,
        DemosaicAlgorithm::PPG,
        DemosaicAlgorithm::AHD,
        DemosaicAlgorithm::DCB,
    ];
    
    for file in &test_files {
        let path = PathBuf::from(file);
        if !path.exists() {
            println!("Skipping benchmark for non-existent file: {}", file);
            continue;
        }
        
        let raw_processor = RawProcessor::new();
        let raw_image = match raw_processor.load_file(&path) {
            Ok(img) => img,
            Err(e) => {
                println!("Error loading RAW file {}: {}", file, e);
                continue;
            }
        };
        
        // Default processing parameters
        let mut params = RawProcessingParams::default();
        
        for algorithm in &algorithms {
            params.demosaic_algorithm = *algorithm;
            
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", algorithm), file), 
                &params, 
                |b, params| {
                    b.iter(|| {
                        let processed = raw_processor.process_image(&raw_image, black_box(params));
                        black_box(processed)
                    })
                }
            );
        }
    }
    
    group.finish();
}

fn bench_raw_noise_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("RAWNoiseReduction");
    
    let test_file = "test_data/raw/test_image_medium.raw";
    let path = PathBuf::from(test_file);
    
    if !path.exists() {
        println!("Skipping noise reduction benchmark for non-existent file: {}", test_file);
        return;
    }
    
    let raw_processor = RawProcessor::new();
    let raw_image = match raw_processor.load_file(&path) {
        Ok(img) => img,
        Err(e) => {
            println!("Error loading RAW file {}: {}", test_file, e);
            return;
        }
    };
    
    // Default processing parameters with different noise reduction settings
    let mut params = RawProcessingParams::default();
    params.demosaic_algorithm = DemosaicAlgorithm::Bilinear; // Use faster algorithm for benchmarking
    
    let noise_params = [
        NoiseReductionParams {
            luminance_strength: 0.0,
            color_strength: 0.0,
            enable_edge_preserving: false,
        },
        NoiseReductionParams {
            luminance_strength: 0.5,
            color_strength: 0.5,
            enable_edge_preserving: false,
        },
        NoiseReductionParams {
            luminance_strength: 1.0,
            color_strength: 1.0,
            enable_edge_preserving: false,
        },
        NoiseReductionParams {
            luminance_strength: 0.5,
            color_strength: 0.5,
            enable_edge_preserving: true,
        },
        NoiseReductionParams {
            luminance_strength: 1.0,
            color_strength: 1.0,
            enable_edge_preserving: true,
        },
    ];
    
    for (i, noise_param) in noise_params.iter().enumerate() {
        params.noise_reduction = noise_param.clone();
        
        group.bench_with_input(
            BenchmarkId::new("NoiseReduction", format!("level={}", i)), 
            &params, 
            |b, params| {
                b.iter(|| {
                    let processed = raw_processor.process_image(&raw_image, black_box(params));
                    black_box(processed)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_raw_detail_enhancement(c: &mut Criterion) {
    let mut group = c.benchmark_group("RAWDetailEnhancement");
    
    let test_file = "test_data/raw/test_image_medium.raw";
    let path = PathBuf::from(test_file);
    
    if !path.exists() {
        println!("Skipping detail enhancement benchmark for non-existent file: {}", test_file);
        return;
    }
    
    let raw_processor = RawProcessor::new();
    let raw_image = match raw_processor.load_file(&path) {
        Ok(img) => img,
        Err(e) => {
            println!("Error loading RAW file {}: {}", test_file, e);
            return;
        }
    };
    
    // Default processing parameters with different detail enhancement settings
    let mut params = RawProcessingParams::default();
    params.demosaic_algorithm = DemosaicAlgorithm::Bilinear; // Use faster algorithm for benchmarking
    params.noise_reduction.luminance_strength = 0.0; // Disable noise reduction
    params.noise_reduction.color_strength = 0.0;
    
    let detail_params = [
        DetailEnhancementParams {
            sharpening: 0.0,
            clarity: 0.0,
            structure: 0.0,
        },
        DetailEnhancementParams {
            sharpening: 0.5,
            clarity: 0.0,
            structure: 0.0,
        },
        DetailEnhancementParams {
            sharpening: 1.0,
            clarity: 0.0,
            structure: 0.0,
        },
        DetailEnhancementParams {
            sharpening: 0.5,
            clarity: 0.5,
            structure: 0.0,
        },
        DetailEnhancementParams {
            sharpening: 0.5,
            clarity: 0.5,
            structure: 0.5,
        },
        DetailEnhancementParams {
            sharpening: 1.0,
            clarity: 1.0,
            structure: 1.0,
        },
    ];
    
    for (i, detail_param) in detail_params.iter().enumerate() {
        params.detail_enhancement = detail_param.clone();
        
        group.bench_with_input(
            BenchmarkId::new("DetailEnhancement", format!("level={}", i)), 
            &params, 
            |b, params| {
                b.iter(|| {
                    let processed = raw_processor.process_image(&raw_image, black_box(params));
                    black_box(processed)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_raw_white_balance(c: &mut Criterion) {
    let mut group = c.benchmark_group("RAWWhiteBalance");
    
    let test_file = "test_data/raw/test_image_medium.raw";
    let path = PathBuf::from(test_file);
    
    if !path.exists() {
        println!("Skipping white balance benchmark for non-existent file: {}", test_file);
        return;
    }
    
    let raw_processor = RawProcessor::new();
    let raw_image = match raw_processor.load_file(&path) {
        Ok(img) => img,
        Err(e) => {
            println!("Error loading RAW file {}: {}", test_file, e);
            return;
        }
    };
    
    // Default processing parameters with different white balance settings
    let mut params = RawProcessingParams::default();
    params.demosaic_algorithm = DemosaicAlgorithm::Bilinear; // Use faster algorithm for benchmarking
    
    let white_balance_settings = [
        WhiteBalance::AsShot,
        WhiteBalance::Auto,
        WhiteBalance::Daylight,
        WhiteBalance::Cloudy,
        WhiteBalance::Shade,
        WhiteBalance::Tungsten,
        WhiteBalance::Fluorescent,
        WhiteBalance::Flash,
        WhiteBalance::Custom { temperature: 5500, tint: 0 },
        WhiteBalance::Custom { temperature: 3200, tint: -10 },
        WhiteBalance::Custom { temperature: 7500, tint: 15 },
    ];
    
    for wb in &white_balance_settings {
        params.white_balance = wb.clone();
        
        group.bench_with_input(
            BenchmarkId::new("WhiteBalance", format!("{:?}", wb)), 
            &params, 
            |b, params| {
                b.iter(|| {
                    let processed = raw_processor.process_image(&raw_image, black_box(params));
                    black_box(processed)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_raw_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("RAWFullPipeline");
    
    let test_files = [
        "test_data/raw/test_image_small.raw",
        "test_data/raw/test_image_medium.raw",
    ];
    
    for file in &test_files {
        let path = PathBuf::from(file);
        if !path.exists() {
            println!("Skipping benchmark for non-existent file: {}", file);
            continue;
        }
        
        let raw_processor = RawProcessor::new();
        let raw_image = match raw_processor.load_file(&path) {
            Ok(img) => img,
            Err(e) => {
                println!("Error loading RAW file {}: {}", file, e);
                continue;
            }
        };
        
        // Complete processing pipeline with high-quality settings
        let high_quality_params = RawProcessingParams {
            demosaic_algorithm: DemosaicAlgorithm::AHD,
            white_balance: WhiteBalance::Auto,
            exposure_compensation: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            noise_reduction: NoiseReductionParams {
                luminance_strength: 0.5,
                color_strength: 0.5,
                enable_edge_preserving: true,
            },
            detail_enhancement: DetailEnhancementParams {
                sharpening: 0.5,
                clarity: 0.3,
                structure: 0.2,
            },
            chromatic_aberration_correction: true,
            lens_distortion_correction: true,
            output_color_space: rust_photo::core::document::ColorSpace::SRGB,
        };
        
        // Fast processing pipeline with low-quality settings
        let fast_params = RawProcessingParams {
            demosaic_algorithm: DemosaicAlgorithm::Bilinear,
            white_balance: WhiteBalance::AsShot,
            exposure_compensation: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            noise_reduction: NoiseReductionParams {
                luminance_strength: 0.0,
                color_strength: 0.0,
                enable_edge_preserving: false,
            },
            detail_enhancement: DetailEnhancementParams {
                sharpening: 0.0,
                clarity: 0.0,
                structure: 0.0,
            },
            chromatic_aberration_correction: false,
            lens_distortion_correction: false,
            output_color_space: rust_photo::core::document::ColorSpace::SRGB,
        };
        
        group.bench_with_input(
            BenchmarkId::new("HighQuality", path.file_name().unwrap().to_str().unwrap()), 
            &high_quality_params, 
            |b, params| {
                b.iter(|| {
                    let processed = raw_processor.process_image(&raw_image, black_box(params));
                    black_box(processed)
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("FastPreview", path.file_name().unwrap().to_str().unwrap()), 
            &fast_params, 
            |b, params| {
                b.iter(|| {
                    let processed = raw_processor.process_image(&raw_image, black_box(params));
                    black_box(processed)
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_raw_demosaic,
    bench_raw_noise_reduction,
    bench_raw_detail_enhancement,
    bench_raw_white_balance,
    bench_raw_full_pipeline
);
criterion_main!(benches); 