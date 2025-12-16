//! Cryptographic operations command implementation.

use std::fs;
use std::io::{self, Read, Write};

use miette::{IntoDiagnostic, Result};

use veil_crypto::{decrypt_raw, encrypt, hash, EncryptionConfig, HashConfig};

use crate::cli::{CryptoAction, CryptoArgs};

/// Run the crypto command.
pub fn run(args: CryptoArgs, _quiet: bool, _json: bool) -> Result<()> {
    match args.action {
        CryptoAction::Encrypt {
            input,
            text,
            output,
            key,
        } => run_encrypt(input, text, output, key),

        CryptoAction::Decrypt {
            input,
            text,
            output,
            key,
        } => run_decrypt(input, text, output, key),

        CryptoAction::Keygen { output } => run_keygen(output),

        CryptoAction::Hash { input, algorithm } => run_hash(input, algorithm),
    }
}

fn run_encrypt(
    input: Option<std::path::PathBuf>,
    text: Option<String>,
    output: Option<std::path::PathBuf>,
    key_path: std::path::PathBuf,
) -> Result<()> {
    // Load key
    let key = fs::read(&key_path).into_diagnostic()?;
    if key.len() != 32 {
        return Err(miette::miette!(
            "Key must be exactly 32 bytes (256 bits), got {} bytes",
            key.len()
        ));
    }

    // Get data to encrypt
    let data = match (input, text) {
        (Some(path), _) => fs::read(&path).into_diagnostic()?,
        (_, Some(t)) => t.into_bytes(),
        _ => {
            // Read from stdin
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf).into_diagnostic()?;
            buf
        }
    };

    // Encrypt
    let config = EncryptionConfig::new(key);
    let result = encrypt(&data, &config).map_err(|e| miette::miette!("Encryption error: {}", e))?;

    // Output (result.value is already base64 encoded)
    if let Some(out_path) = output {
        // Write base64 encoded ciphertext to file
        fs::write(&out_path, &result.value).into_diagnostic()?;
        eprintln!("Encrypted data written to: {}", out_path.display());
    } else {
        // Output base64 to stdout
        println!("{}", result.value);
    }

    Ok(())
}

fn run_decrypt(
    input: Option<std::path::PathBuf>,
    text: Option<String>,
    output: Option<std::path::PathBuf>,
    key_path: std::path::PathBuf,
) -> Result<()> {
    // Load key
    let key = fs::read(&key_path).into_diagnostic()?;
    if key.len() != 32 {
        return Err(miette::miette!(
            "Key must be exactly 32 bytes (256 bits), got {} bytes",
            key.len()
        ));
    }

    // Get encoded ciphertext (base64 string)
    let encoded = match (input, text) {
        (Some(path), _) => {
            // Read file as string (base64 encoded ciphertext)
            fs::read_to_string(&path).into_diagnostic()?
        }
        (_, Some(t)) => t,
        _ => {
            // Read from stdin
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).into_diagnostic()?;
            buf.trim().to_string()
        }
    };

    // Decrypt using decrypt_raw which takes encoded string
    let config = EncryptionConfig::new(key);
    let result =
        decrypt_raw(&encoded, &config).map_err(|e| miette::miette!("Decryption error: {}", e))?;

    // Output
    if let Some(out_path) = output {
        fs::write(&out_path, &result).into_diagnostic()?;
        eprintln!("Decrypted data written to: {}", out_path.display());
    } else {
        io::stdout().write_all(&result).into_diagnostic()?;
    }

    Ok(())
}

fn run_keygen(output: std::path::PathBuf) -> Result<()> {
    use rand::RngCore;

    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    fs::write(&output, key).into_diagnostic()?;
    eprintln!("Generated 256-bit key: {}", output.display());
    eprintln!("Keep this key secure! It is required for decryption.");

    Ok(())
}

fn run_hash(input: String, algorithm: String) -> Result<()> {
    let config = match algorithm.to_lowercase().as_str() {
        "sha256" | "sha-256" => HashConfig::sha256(),
        "sha512" | "sha-512" => HashConfig::sha512(),
        _ => return Err(miette::miette!("Unsupported algorithm: {}", algorithm)),
    };

    // Check if input is a file path
    let data = if std::path::Path::new(&input).exists() {
        fs::read(&input).into_diagnostic()?
    } else {
        input.into_bytes()
    };

    let result = hash(&data, &config).map_err(|e| miette::miette!("Hash error: {}", e))?;

    println!("{}", result.value);

    Ok(())
}
