use image::DynamicImage;
use std::fmt::Debug;
use std::path::PathBuf;

pub mod database;

const CHUNK_SIZE: usize = 5;

#[derive(Debug, Default)]
pub struct Server {
    sources: Option<Vec<Source>>,
    current_source: usize,
    current_source_pages: Option<PDFWithDebug>,
    page_count: usize,
}

impl Server {
    pub fn new() -> Self {
        Self {
            sources: None,
            current_source: 0,
            current_source_pages: None,
            page_count: 0,
        }
    }

    pub fn set_sources(&mut self, sources: Vec<Source>) {
        // TODO: Sort sources by name A-z
        self.sources = Some(sources);
    }

    // pub fn get_chunk(&self) -> Option<Vec<DynamicImage>> {
    //     match self.sources {
    //         Some() => {}
    //         None => None,
    //     }
    // }
}

#[derive(Debug, Default)]
pub enum SourceType {
    #[default]
    PDF,
    ImageSequence,
    Directory,
}

/// Wrapper struct that implements fmt::Debug for pdf2image::PDF
struct PDFWithDebug {
    #[allow(dead_code)]
    pdf: pdf2image::PDF,
}
impl PDFWithDebug {
    pub fn new(path: &PathBuf) -> Self {
        let pdf = pdf2image::PDF::from_file(&path).unwrap();
        Self { pdf }
    }

    pub fn page_count(&self) -> usize {
        self.pdf.page_count() as usize
    }
}
impl Debug for PDFWithDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PDFWithDebug").finish()
    }
}

#[derive(Debug, Default)]
pub struct Source {
    source_type: SourceType,
    path: PathBuf,
    current_page: usize,
    page_count: usize,
}
impl Source {
    pub fn new(source_type: SourceType, path: PathBuf) -> Self {
        let pages: Option<PDFWithDebug>;
        let mut page_count: usize = 0;
        match source_type {
            SourceType::PDF => {
                let t_pages = PDFWithDebug::new(&path);
                page_count = t_pages.page_count();
                pages = Some(t_pages);
            }
            _ => {
                pages = None;
            }
        }

        Self {
            source_type,
            path,
            current_page: 0,
            page_count,
        }
    }
}
