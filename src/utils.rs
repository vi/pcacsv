use std::{collections::BTreeSet, str::FromStr};



#[derive(Debug,Default)]
pub struct ColumnsSpecifier(pub BTreeSet<usize>);
#[derive(Debug)]
pub struct DelimiterSpecifier(pub u8);

impl FromStr for ColumnsSpecifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = number_range::NumberRange::default();
        Ok(ColumnsSpecifier(p.parse_str(s)?.collect()))
    }
}

impl FromStr for DelimiterSpecifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_ascii() && s.len() == 1 {
            Ok(DelimiterSpecifier(s.as_bytes()[0]))
        } else {
            Err(anyhow::anyhow!("Delimiter should be exactly one ASCII character"))
        }
    }
}

impl crate::MyOptions {
    pub fn get_csv_reader(&self) -> csv::ReaderBuilder {
        let mut b = csv::ReaderBuilder::new();
        if self.no_header {
            b.has_headers(false);
        }
        if let Some(ref x) = self.delimiter {
            b.delimiter(x.0);
        }
        if let Some(ref x) = self.record_delimiter {
            b.terminator(csv::Terminator::Any(x.0));
        }
        b
    }
    pub fn get_csv_writer(&self) -> csv::WriterBuilder {
        let mut b = csv::WriterBuilder::new();
        if self.no_header || self.no_output_header {
            b.has_headers(false);
        }
        if let Some(ref x) = self.delimiter {
            b.delimiter(x.0);
        }
        if let Some(ref x) = self.record_delimiter {
            b.terminator(csv::Terminator::Any(x.0));
        }
        b
    }

    pub fn get_istream(&self) -> anyhow::Result<Box<dyn std::io::Read>> {
        if let Some(ref f) = self.input_path {
            Ok(Box::new(std::fs::File::open(f)?))
        } else {
            Ok(Box::new(std::io::stdin()))
        }
    }

    pub fn get_ostream(&self) -> anyhow::Result<Box<dyn std::io::Write>> {
        if let Some(ref f) = self.output {
            Ok(Box::new(std::fs::File::create(f)?))
        } else {
            Ok(Box::new(std::io::stdout()))
        }
    }
}
