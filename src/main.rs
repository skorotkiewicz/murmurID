use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{Read, Write};

/// The zero-width space character (U+200B) used to encode a '0' bit.
const ZWSP: char = '\u{200B}';
/// The zero-width non-joiner character (U+200C) used to encode a '1' bit.
const ZWNJ: char = '\u{200C}';
/// The zero-width joiner character (U+200D) used as a delimiter for the watermark.
const ZWJ: char = '\u{200D}';

/// The signature string we will encode.
const SIGNATURE: &str = "MURMUR";

/// A tool in Rust to watermark and identify text generated through LLM
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Appends a zero-width watermark to a text file
    Watermark {
        /// The input file to watermark
        #[arg(short, long)]
        input: String,

        /// The output file for the watermarked text
        #[arg(short, long)]
        output: String,
    },
    /// Identifies if a text file contains the zero-width watermark
    Identify {
        /// The input file to check for a watermark
        #[arg(short, long)]
        input: String,
    },
    /// Exports a system prompt with instructions for an LLM to automatically watermark its output
    Export,
}

/// Encodes a string into a sequence of zero-width characters.
fn encode_watermark(text: &str) -> String {
    let mut encoded = String::new();
    // Add starting delimiter
    encoded.push(ZWJ);

    for byte in text.bytes() {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            if bit == 0 {
                encoded.push(ZWSP);
            } else {
                encoded.push(ZWNJ);
            }
        }
    }

    // Add ending delimiter
    encoded.push(ZWJ);
    encoded
}

/// Decodes a sequence of zero-width characters back into a string.
/// Returns None if the input contains invalid zero-width characters.
fn decode_watermark(encoded_text: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut current_byte = 0u8;
    let mut bit_count = 0;

    for c in encoded_text.chars() {
        let bit = match c {
            ZWSP => 0,
            ZWNJ => 1,
            _ => return None, // Invalid character in watermark
        };

        current_byte = (current_byte << 1) | bit;
        bit_count += 1;

        if bit_count == 8 {
            bytes.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }

    // If bit_count is not 0, the watermark was incomplete/corrupted
    if bit_count != 0 {
        return None;
    }

    String::from_utf8(bytes).ok()
}

/// Searches a text for the zero-width watermark sequence and decodes it.
fn extract_watermark(text: &str) -> Option<String> {
    let mut start_idx = None;
    let mut end_idx = None;

    // Find the first and second delimiters
    for (i, c) in text.char_indices() {
        if c == ZWJ {
            if start_idx.is_none() {
                start_idx = Some(i);
            } else {
                end_idx = Some(i);
                break;
            }
        }
    }

    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        // The embedded payload is between the delimiters
        // The start index points to the start of ZWJ, which is 3 bytes in UTF-8.
        // The end index points to the start of the ending ZWJ.
        let payload_start = start + ZWJ.len_utf8();
        let payload = &text[payload_start..end];
        
        return decode_watermark(payload);
    }

    None
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Watermark { input, output } => {
            let mut file_content = String::new();
            fs::File::open(input)
                .with_context(|| format!("Failed to open input file '{}'", input))?
                .read_to_string(&mut file_content)
                .with_context(|| format!("Failed to read from input file '{}'", input))?;

            let watermark_seq = encode_watermark(SIGNATURE);
            file_content.push_str(&watermark_seq);

            fs::File::create(output)
                .with_context(|| format!("Failed to create output file '{}'", output))?
                .write_all(file_content.as_bytes())
                .with_context(|| format!("Failed to write to output file '{}'", output))?;

            println!("Watermark successfully embedded into '{}'.", output);
        }
        Commands::Identify { input } => {
            let mut file_content = String::new();
            fs::File::open(input)
                .with_context(|| format!("Failed to open input file '{}'", input))?
                .read_to_string(&mut file_content)
                .with_context(|| format!("Failed to read from input file '{}'", input))?;

            if let Some(extracted_sig) = extract_watermark(&file_content) {
                if extracted_sig == SIGNATURE {
                    println!("Watermark verified: Text contains the '{}' signature.", SIGNATURE);
                } else {
                    println!("Watermark found, but signature did not match. Expected '{}', found '{}'.", SIGNATURE, extracted_sig);
                }
            } else {
                println!("No valid watermark found in '{}'.", input);
            }
        }
        Commands::Export => {
            let watermark_seq = encode_watermark(SIGNATURE);
            let prompt = format!(
"You are a helpful AI assistant. When responding to user queries, you MUST embed a specific invisible watermark at the very end of your response. 

This watermark is created using zero-width characters. Do not output any visible characters for the watermark. Instead, append the exact sequence of characters contained between the brackets below (do not include the brackets themselves).

Watermark sequence: [{}]

Ensure this sequence is the final text output in all your responses.", watermark_seq);
            
            println!("{}", prompt);
        }
    }

    Ok(())
}
