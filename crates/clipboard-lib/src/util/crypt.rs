use anyhow::Context;
use crypto_common::Generate;
use std::io::Write;
use tracing::trace;

#[cfg(feature = "encrypt_chacha20poly1305")]
type K = chacha20poly1305::Key;
#[cfg(all(not(feature = "encrypt_chacha20poly1305"), feature = "encrypt_aes_gcm"))]
type K = aes_gcm::Key<aes_gcm::Aes256Gcm>;

pub fn generate_new_key() -> anyhow::Result<Vec<u8>> {
    K::try_generate()
        .map_err(|_| anyhow::anyhow!("Failed to generate new encryption key"))
        .map(|k| k.to_vec())
}

pub struct SecretEncryptWriter<W: Write> {
    key: Vec<u8>,
    buffer: Vec<u8>,
    writer: W,
    config: Config,
}

pub enum Config {
    #[cfg(feature = "encrypt_chacha20poly1305")]
    ChaCha20Poly1305,
    #[cfg(feature = "encrypt_aes_gcm")]
    AesGcm,
}

impl<W: Write> SecretEncryptWriter<W> {
    pub const fn new(writer: W, key: Vec<u8>, config: Config) -> Self {
        Self {
            buffer: Vec::new(),
            key,
            writer,
            config,
        }
    }

    pub fn encrypt(&self, cleartext: &[u8]) -> anyhow::Result<Vec<u8>> {
        trace!("length of cleartext: {}", cleartext.len());
        match self.config {
            #[cfg(feature = "encrypt_chacha20poly1305")]
            Config::ChaCha20Poly1305 => {
                use chacha20poly1305::{KeyInit, aead::Aead};
                let nonce = chacha20poly1305::Nonce::try_generate()
                    .map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;
                let cypher = chacha20poly1305::ChaCha20Poly1305::new_from_slice(&self.key)
                    .context("Failed to generate cypher with encryption key")?;
                let obsf = cypher
                    .encrypt(&nonce, cleartext)
                    .context("Encryption failed")?;
                let mut nonce = nonce.to_vec();
                nonce.extend_from_slice(&obsf);
                Ok(nonce)
            }
            #[cfg(feature = "encrypt_aes_gcm")]
            Config::AesGcm => {
                use aes_gcm::{KeyInit, aead::Aead};
                let nonce = aes_gcm::Nonce::try_generate()
                    .map_err(|_| anyhow::anyhow!("Failed to generate nonce"))?;
                let cypher = aes_gcm::Aes256Gcm::new_from_slice(&self.key)
                    .context("Failed to generate cypher with encryption key")?;
                let obsf = cypher
                    .encrypt(&nonce, cleartext)
                    .context("Encryption failed")?;
                let mut nonce = nonce.to_vec();
                nonce.extend_from_slice(&obsf);
                Ok(nonce)
            }
        }
    }
}

impl<W: Write> Write for SecretEncryptWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> Drop for SecretEncryptWriter<W> {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            if let Ok(encrypted) = self.encrypt(&self.buffer) {
                trace!("Writing {} bytes encrypted data to writer", encrypted.len());
                let _ = self.writer.write_all(&encrypted);
            }
            self.buffer.clear();
        }
        let _ = self.writer.flush();
    }
}

// pub fn decrypt(&self, obsf: &[u8]) -> anyhow::Result<Vec<u8>> {
//     type NonceSize = <ChaCha20Poly1305 as AeadCore>::NonceSize;
//     let cipher = generate_cypher(&self.key).context("Failed to generate cipher")?;
//     let (nonce_bytes, ciphertext) = obsf.split_at(NonceSize::to_usize());
//     let nonce =
//         Nonce::<ChaCha20Poly1305>::try_from(nonce_bytes).context("Failed to parse nonce")?;
//     let out = cipher
//         .decrypt(&nonce, ciphertext)
//         .context("Decryption failed")?;
//     Ok(out)
// }
