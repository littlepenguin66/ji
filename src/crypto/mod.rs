pub mod age;
pub mod pgp;

pub trait Cipher {
    fn encrypt(data: &[u8], recipients: &[String]) -> Result<Vec<u8>, crate::error::Error>;
    fn decrypt(data: &[u8]) -> Result<Vec<u8>, crate::error::Error>;
    fn list_recipients(data: &[u8]) -> Result<Vec<String>, crate::error::Error>;
}
