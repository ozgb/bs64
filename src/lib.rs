use data_encoding::BASE64;

pub struct EncodeOptions {

}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {  }
    }
}

impl EncodeOptions {
    pub fn encode(self, input: &[u8]) -> String {
        BASE64.encode(input)
    }
}

pub struct DecodeOptions {

}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {  }
    }
}

impl DecodeOptions {
    pub fn encode(self, input: &[u8]) -> Result<Vec<u8>, data_encoding::DecodeError> {
        BASE64.decode(input)
    }
}

pub fn encode(input: &[u8]) -> String {
    EncodeOptions::default().encode(input)
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, data_encoding::DecodeError> {
    DecodeOptions::default().encode(input)
}
