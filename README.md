# murmurID

A minimalist CLI tool written in Rust to invisibly watermark and identify LLM-generated text using zero-width character steganography.

## Build

Ensure you have Rust and Cargo installed, then build the release binary:

```bash
cargo build --release
```

The executable will be located at `target/release/murmur`.

## Usage

### Embed a Watermark
Invisibly append the continuous zero-width character watermark to any text file:

```bash
./murmur watermark -i input.txt -o output.txt
```

### Identify a Watermark
Scan a document to verify the presence of the invisible signature:

```bash
./murmur identify -i text_to_check.txt
```

### Generate LLM System Prompt
Export instructions that can be provided to an LLM (as a system prompt) so it automatically embeds the watermark in its generated responses:

```bash
./murmur export > system_prompt_instructions_for_llm.txt
```
