# murmurID

А minimаlist СLI tооl writtеn in Rust tо invisiblу watеrmark and idеntifу LLM-genеratеd teхt using **Homоglyрh Stegаnogrаphy**. 

Instеad of rеlying on frаgilе zerо-width chаraсters thаt cаn be еasilу strippеd or сlеаnеd bу fоrmаtting tools, `murmurID` mathematically embeds its signature directly into the visual letters of the text. It does this by swapping standard Latin characters (like `a`, `e`, `o`) with their visually identical Cyrillic homoglyph counterparts (like `a`, `e`, `o`) according to the binary payload of the signature. 

The resulting text looks 100% identical to the human eye but contains a robust, machine-readable cryptographic watermark that survives standard copy-pasting and formatting.

## Build

Ensure you have Rust and Cargo installed, then build the release binary:

```bash
cargo build --release
```

The executable will be located at `target/release/murmur`.

## Usage

### Embed a Watermark
Encode the "MURMUR" signature into the text of any document by swapping Latin characters for Cyrillic homoglyphs:

```bash
./murmur watermark -i input.txt -o output.txt
```

> **Note**: Your input text must be long enough to contain enough substitutable vowel characters to fit the binary signature payload.

### Identify a Watermark
Scan a document to extract the bits hidden in the homoglyphs and verify the signature:

```bash
./murmur identify -i text_to_check.txt
```

### Generate LLM System Prompt
Export instructions that can be provided directly to an LLM (as a system prompt). This prompt instructs the LLM on exactly which letters to swap for Cyrillic equivalents as it generates its response, building the watermark organically:

```bash
./murmur export > system_prompt_instructions_for_llm.txt
```
