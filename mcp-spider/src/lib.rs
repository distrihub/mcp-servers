mod server;
mod scraper_tools;

pub use server::build;
pub use scraper_tools::{
    ElementExtractor, SpiderSession, WebAutomation, XPathAlternative,
    ScrapingOptions, ScrapingResult
};

#[cfg(test)]
mod tests;
