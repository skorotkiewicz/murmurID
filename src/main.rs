use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};

/// The signature string we will encode.
const SIGNATURE: &str = "MURMUR";

const START_MARKER: u8 = 0xFF;
const END_MARKER: u8 = 0xFE;

/// A tool in Rust to watermark and identify text generated through LLM using Homoglyphs
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Appends a homoglyph watermark to a text file
    Watermark {
        /// The input file to watermark
        #[arg(short, long)]
        input: String,

        /// The output file for the watermarked text
        #[arg(short, long)]
        output: String,
    },
    /// Identifies if a text file contains the homoglyph watermark
    Identify {
        /// The input file to check for a watermark
        #[arg(short, long)]
        input: String,
    },
    /// Exports instructions for LLMs to generate text with the watermark
    Export,
}

lazy_static::lazy_static! {
    static ref LATIN_TO_CYRILLIC: HashMap<char, char> = {
        let mut m = HashMap::new();
        m.insert('a', 'а'); // U+0430
        m.insert('c', 'с'); // U+0441
        m.insert('e', 'е'); // U+0435
        m.insert('o', 'о'); // U+043E
        m.insert('p', 'р'); // U+0440
        m.insert('x', 'х'); // U+0445
        m.insert('y', 'у'); // U+0443
        m.insert('A', 'А'); // U+0410
        m.insert('C', 'С'); // U+0421
        m.insert('E', 'Е'); // U+0415
        m.insert('O', 'О'); // U+041E
        m.insert('P', 'Р'); // U+0420
        m.insert('X', 'Х'); // U+0425
        m.insert('Y', 'У'); // U+0423
        m
    };

    static ref CYRILLIC_TO_LATIN: HashMap<char, char> = {
        let mut m = HashMap::new();
        for (&latin, &cyrillic) in LATIN_TO_CYRILLIC.iter() {
            m.insert(cyrillic, latin);
        }
        m
    };
}

/// Helper to get a bit stream from a byte slice
fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::new();
    for byte in bytes {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1 == 1);
        }
    }
    bits
}

fn encode_homoglyph_watermark(text: &str, payload: &str) -> String {
    let mut bytes_to_encode = Vec::new();
    bytes_to_encode.push(START_MARKER);
    bytes_to_encode.extend_from_slice(payload.as_bytes());
    bytes_to_encode.push(END_MARKER);

    let bits = bytes_to_bits(&bytes_to_encode);
    let mut bit_idx = 0;
    
    let mut watermarked_text = String::with_capacity(text.len());

    for c in text.chars() {
        if LATIN_TO_CYRILLIC.contains_key(&c) {
            let bit = bits[bit_idx % bits.len()];
            bit_idx += 1;
            
            if bit {
                watermarked_text.push(*LATIN_TO_CYRILLIC.get(&c).unwrap());
            } else {
                watermarked_text.push(c);
            }
        } else if CYRILLIC_TO_LATIN.contains_key(&c) {
            let bit = bits[bit_idx % bits.len()];
            bit_idx += 1;
            if bit {
                watermarked_text.push(c);
            } else {
                watermarked_text.push(*CYRILLIC_TO_LATIN.get(&c).unwrap());
            }
        } else {
            watermarked_text.push(c);
        }
    }

    watermarked_text
}

fn extract_homoglyph_watermark(text: &str) -> Option<String> {
    let mut bits = Vec::new();

    // Extract all bits from the mapped characters
    for c in text.chars() {
        if LATIN_TO_CYRILLIC.contains_key(&c) {
            bits.push(false);
        } else if CYRILLIC_TO_LATIN.contains_key(&c) {
            bits.push(true);
        }
    }

    // Convert START_MARKER to boolean array for easy comparison
    let start_bits = bytes_to_bits(&[START_MARKER]);

    // Use a sliding window to find the start marker bit sequence
    let mut i = 0;
    while i + start_bits.len() <= bits.len() {
        let mut is_start = true;
        for j in 0..start_bits.len() {
            if bits[i + j] != start_bits[j] {
                is_start = false;
                break;
            }
        }

        if is_start {
            // We found a start marker. Now read 8 bits at a time aligned to this offset.
            let mut payload_bytes = Vec::new();
            let mut bit_idx = i + start_bits.len();
            let mut found_end = false;

            while bit_idx + 8 <= bits.len() {
                let mut current_byte = 0u8;
                for j in 0..8 {
                    let bit_val = if bits[bit_idx + j] { 1 } else { 0 };
                    current_byte = (current_byte << 1) | bit_val;
                }

                if current_byte == END_MARKER {
                    found_end = true;
                    break;
                } else if current_byte == START_MARKER {
                    // Start marker inside payload, probably false synchronization. 
                    // Let's abort and keep looking.
                    break;
                }

                payload_bytes.push(current_byte);
                bit_idx += 8;
            }

            if found_end {
                if let Ok(sig) = String::from_utf8(payload_bytes) {
                    return Some(sig);
                }
            }
        }
        i += 1;
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

            let watermarked = encode_homoglyph_watermark(&file_content, SIGNATURE);

            fs::File::create(output)
                .with_context(|| format!("Failed to create output file '{}'", output))?
                .write_all(watermarked.as_bytes())
                .with_context(|| format!("Failed to write to output file '{}'", output))?;

            // We can check if it actually fit
            let required_chars = (SIGNATURE.len() + 2) * 8; // Including markers
            let mut available_chars = 0;
            for c in file_content.chars() {
                if LATIN_TO_CYRILLIC.contains_key(&c) || CYRILLIC_TO_LATIN.contains_key(&c) {
                    available_chars += 1;
                }
            }

            if available_chars < required_chars {
                println!("Warning: Not enough substitutable characters in text to embed the full watermark.");
                println!("Required: {}, Available: {}", required_chars, available_chars);
                println!("The watermark may be unrecoverable or corrupted.");
            } else {
                println!("Watermark successfully embedded into '{}'.", output);
            }
        }
        Commands::Identify { input } => {
            let mut file_content = String::new();
            fs::File::open(input)
                .with_context(|| format!("Failed to open input file '{}'", input))?
                .read_to_string(&mut file_content)
                .with_context(|| format!("Failed to read from input file '{}'", input))?;

            if let Some(extracted_sig) = extract_homoglyph_watermark(&file_content) {
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
            let mut bytes_to_encode = Vec::new();
            bytes_to_encode.push(START_MARKER);
            bytes_to_encode.extend_from_slice(SIGNATURE.as_bytes());
            bytes_to_encode.push(END_MARKER);

            let bits = bytes_to_bits(&bytes_to_encode);
            let bit_string: String = bits.iter().map(|&b| if b { '1' } else { '0' }).collect();

            let prompt = format!(
"You are a helpful AI assistant. When responding to user queries, you MUST embed a specific invisible watermark directly into the shape of the letters you output. 

You must strictly replace standard Latin letters with their visually identical Cyrillic homoglyph counterparts in order from the start of your response, according to the binary string below. Each binary bit maps to a substitutable letter (a, c, e, o, p, x, y, A, C, E, O, P, X, Y). 
0 = keep standard Latin letter
1 = replace with Cyrillic homoglyph

Here is the exact mapping of Latin character -> Cyrillic homoglyph you must use:
- `a` -> `а`
- `c` -> `с`
- `e` -> `е`
- `o` -> `о`
- `p` -> `р`
- `x` -> `х`
- `y` -> `у`
- `A` -> `А`
- `C` -> `С`
- `E` -> `Е`
- `O` -> `О`
- `P` -> `Р`
- `X` -> `Х`
- `Y` -> `У`

Ensure that you embed this sequence continually throughout the text. When you reach the end of the bitstream sequence, START OVER FROM THE BEGINNING of the sequence and continue mapping substitutions over and over again until the text ends.

Binary Watermark Bitstream (REPEAT THIS OVER AND OVER):
{}

Do not output the binary sequence itself, just modify the text you naturally generate.",
                bit_string
            );
            
            println!("{}", prompt);
        }
    }

    Ok(())
}
