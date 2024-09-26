use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use brotli::CompressorWriter;
use bzip2::write::BzEncoder;
use xz2::write::XzEncoder;
use std::fs;

enum CompressionType {
    Zlib,
    Gzip,
    Brotli,
    Bzip2,
    Xz,
}

fn compress_file(input_path: &str, output_path: &str, compression_type: CompressionType) -> io::Result<()> {
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' not found.", input_path);
        return Err(io::Error::new(io::ErrorKind::NotFound, "Input file not found"));
    }
    
    let mut input_file = File::open(input_path)?;
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;

    let original_size = input_data.len();
    println!("Original file size: {:.2} MB", original_size as f64 / (1024.0 * 1024.0));

    let output_file = File::create(output_path)?;

    match compression_type {
        CompressionType::Zlib => {
            let compression = Compression::default();
            let mut encoder = ZlibEncoder::new(output_file, compression);
            encoder.write_all(&input_data)?;
            encoder.finish()?;
        }
        CompressionType::Gzip => {
            let compression = Compression::default();
            let mut encoder = GzEncoder::new(output_file, compression);
            encoder.write_all(&input_data)?;
            encoder.finish()?;
        }
        CompressionType::Brotli => {
            let mut compressor = CompressorWriter::new(output_file, 4096, 5, 22); // Brotli level 5
            compressor.write_all(&input_data)?;
            compressor.flush()?;
        }
        CompressionType::Bzip2 => {
            let mut encoder = BzEncoder::new(output_file, bzip2::Compression::default());
            encoder.write_all(&input_data)?;
            encoder.finish()?;
        }
        CompressionType::Xz => {
            let mut encoder = XzEncoder::new(output_file, 6); // XZ level 6
            encoder.write_all(&input_data)?;
            encoder.finish()?;
        }
    }
    
    let compressed_size = fs::metadata(output_path)?.len();
    println!("Compressed file size: {:.2} MB", compressed_size as f64 / (1024.0 * 1024.0));

    Ok(())
}

fn decompress_file(input_path: &str, output_path: &str, compression_type: CompressionType) -> io::Result<()> {
    if !Path::new(input_path).exists() {
        eprintln!("Error: Compressed file '{}' not found.", input_path);
        return Err(io::Error::new(io::ErrorKind::NotFound, "Compressed file not found"));
    }

    let input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.take(1024 * 1024).read_to_end(&mut buffer)?;

    let mut output_file = File::create(output_path)?;

    match compression_type {
        CompressionType::Zlib => {
            let mut decoder = ZlibDecoder::new(&buffer[..]);
            io::copy(&mut decoder, &mut output_file)?;
        }
        CompressionType::Gzip => {
            let mut decoder = GzDecoder::new(&buffer[..]);
            io::copy(&mut decoder, &mut output_file)?;
        }
        CompressionType::Brotli => {
            let mut decompressed_data = Vec::new();
            brotli::Decompressor::new(&buffer[..], 4096).read_to_end(&mut decompressed_data)?;
            output_file.write_all(&decompressed_data)?;
        }
        CompressionType::Bzip2 => {
            let mut decoder = bzip2::read::BzDecoder::new(&buffer[..]);
            io::copy(&mut decoder, &mut output_file)?;
        }
        CompressionType::Xz => {
            let mut decoder = xz2::read::XzDecoder::new(&buffer[..]);
            io::copy(&mut decoder, &mut output_file)?;
        }
    }
    
    let decompressed_size = fs::metadata(output_path)?.len();
    println!("Decompressed file size: {:.2} MB", decompressed_size as f64 / (1024.0 * 1024.0));

    Ok(())
}

fn main() {
    println!("Do you want to compress or decompress a file? (C/D): ");
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice).expect("Failed to read choice");
    let choice = choice.trim().to_uppercase();

    if choice == "C" {
        // Compression section
        println!("Enter the file to compress: ");
        let mut input_file: String = String::new();
        std::io::stdin().read_line(&mut input_file).expect("Failed to read input file");
        let input_file = input_file.trim();

        println!("Enter the output compressed file name (e.g., compressed_file.gz): ");
        let mut output_file = String::new();
        std::io::stdin().read_line(&mut output_file).expect("Failed to read output file");
        let output_file = output_file.trim();

        println!("Enter the compression type (Zlib/Gzip/Brotli/Bzip2/Xz): ");
        let mut type_input = String::new();
        std::io::stdin().read_line(&mut type_input).expect("Failed to read compression type");
        let compression_type = match type_input.trim().to_lowercase().as_str() {
            "zlib" => CompressionType::Zlib,
            "gzip" => CompressionType::Gzip,
            "brotli" => CompressionType::Brotli,
            "bzip2" => CompressionType::Bzip2,
            "xz" => CompressionType::Xz,
            _ => {
                println!("Invalid compression type! Defaulting to Brotli.");
                CompressionType::Brotli
            }
        };

        println!("Enter a compression level (1 = fastest, 9 = best compression): ");
        let mut level_input = String::new();
        std::io::stdin().read_line(&mut level_input).expect("Failed to read line");
        let _compression_level: u32 = level_input.trim().parse().unwrap_or(9);

        match compress_file(input_file, output_file, compression_type) {
            Ok(_) => println!("File compressed successfully!"),
            Err(e) => eprintln!("Failed to compress the file: {}", e),
        }

    } else if choice == "D" {
        // Decompression section
        println!("Enter the compressed file to decompress (e.g., compressed_file.gz): ");
        let mut compressed_file = String::new();
        std::io::stdin().read_line(&mut compressed_file).expect("Failed to read compressed file");
        let compressed_file = compressed_file.trim();

        println!("Enter the output file name after decompression (e.g., decompressed_file.pdf): ");
        let mut output_file = String::new();
        std::io::stdin().read_line(&mut output_file).expect("Failed to read output file");
        let output_file = output_file.trim();

        println!("Enter the compression type (Zlib/Gzip/Brotli/Bzip2/Xz): ");
        let mut type_input = String::new();
        std::io::stdin().read_line(&mut type_input).expect("Failed to read compression type");
        let compression_type = match type_input.trim().to_lowercase().as_str() {
            "zlib" => CompressionType::Zlib,
            "gzip" => CompressionType::Gzip,
            "brotli" => CompressionType::Brotli,
            "bzip2" => CompressionType::Bzip2,
            "xz" => CompressionType::Xz,
            _ => {
                println!("Invalid compression type! Defaulting to Brotli.");
                CompressionType::Brotli
            }
        };

        match decompress_file(compressed_file, output_file, compression_type) {
            Ok(_) => println!("File decompressed successfully!"),
            Err(e) => eprintln!("Failed to decompress the file: {}", e),
        }

    } else {
        println!("Invalid choice! Please enter 'C' for compression or 'D' for decompression.");
    }
}
