# murmurID

A minimalist, indestructible CLI tool written in Rust to invisibly watermark and identify LLM-generated text using **Homoglyph Steganography**. 

`murmurID` mathematically embeds a hidden signature directly into the visual letters of text by swapping standard Latin characters (like `a`, `e`, `o`) with visually identical Cyrillic homoglyph counterparts. 

Because the watermark is structural, it survives standard copy-pasting, visual inspection, and formatting cleaners (like zero-width stripping tools).

## Build
```bash
cargo build --release
```

## Usage

**1. Watermark Text**  
Embeds the "MURMUR" signature repeatedly into a document.
```bash
./target/release/murmur watermark -i input.txt -o output.txt
```

**2. Identify Watermark**  
Scans a document and mathematically proves if the "MURMUR" signature is hidden inside it.
```bash
./target/release/murmur identify -i text_to_check.txt
```

**3. Generate LLM Prompt**  
Exports instructions to inject into an LLM's system prompt. This forces the LLM to organically generate its responses using the exact homoglyph swaps needed to form the watermark.
```bash
./target/release/murmur export > prompt.txt
```
