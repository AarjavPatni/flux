// src/url_generator.rs

pub struct UrlGenerator {
    count: usize,
}

impl UrlGenerator {
    pub fn new(count: usize) -> Self {
        UrlGenerator { count }
    }

    /// Generate URLs for random images from Lorem Picsum
    /// Format: https://picsum.photos/seed/{i}/800/600
    /// Using seed ensures same images across runs
    pub fn generate(&self) -> Vec<String> {
        let mut urls: Vec<String> = Vec::new();
        for i in 0..self.count {
            urls.push(format!("https://picsum.photos/seed/{}/800/600", i));
        }
        urls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_count() {
        let gen = UrlGenerator::new(10);
        let urls = gen.generate();
        assert_eq!(urls.len(), 10);
    }

    #[test]
    fn urls_have_correct_format() {
        let gen = UrlGenerator::new(5);
        let urls = gen.generate();
        assert!(urls[0].contains("picsum.photos"));
        assert!(urls[0].contains("/800/600"));
    }
}
