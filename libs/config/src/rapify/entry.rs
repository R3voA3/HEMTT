use crate::Entry;

use super::Rapify;

impl Rapify for Entry {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let written = match self {
            Self::Str(s) => s.rapify(output, offset),
            Self::Number(n) => n.rapify(output, offset),
            Self::Array(a) => a.rapify(output, offset),
        }?;
        assert_eq!(written, self.rapified_length());
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::Str(s) => s.rapified_length(),
            Self::Number(n) => n.rapified_length(),
            Self::Array(a) => a.rapified_length(),
        }
    }

    fn rapified_code(&self) -> u8 {
        match self {
            Self::Str(s) => s.rapified_code(),
            Self::Number(n) => n.rapified_code(),
            Self::Array(a) => a.rapified_code(),
        }
    }
}