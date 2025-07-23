use image::DynamicImage;
use log::error;
use pdf2image::{PDF, PDF2ImageError, RenderOptionsBuilder};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::path::PathBuf;

pub mod database;

/// How many extra pages will be store at both ends of the current page\
/// For example, a value of '2' means that the page store will hold up to 5 pages: previous 2, current and next 2.
const EXTRA_PAGES_AT_ENDS: usize = 2;

#[derive(Debug, Default)]
pub struct Server {
    sources: Option<Vec<Source>>,
    source_count: usize,
    /// Index of the current source
    current_source: usize,
    page_count: usize,
    /// Holds a chunk of DynamicImages to be used by the reader.\
    /// A chunk is 5 pages by default (previous two, current and next two).
    page_store: VecDeque<DynamicImage>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            sources: None,
            source_count: 0,
            current_source: 0,
            page_count: 0,
            page_store: VecDeque::with_capacity(EXTRA_PAGES_AT_ENDS * 2 + 1),
        }
    }

    pub fn set_sources(&mut self, sources: Vec<Source>, page_count: usize) {
        // TODO: Sort sources by name A-z
        self.source_count = sources.len();
        self.sources = Some(sources);
        self.current_source = 0;
        self.page_count = page_count;
        self.page_store.clear();

        self.get_chunk_for_page(0, 0);
    }

    pub fn get_prev_page(&mut self) -> Option<&DynamicImage> {
        let mut prev_page_in_store = EXTRA_PAGES_AT_ENDS;

        if self.sources.is_some() {
            let current_source = self.sources.as_mut().unwrap().get_mut(self.current_source);
            if current_source.is_some() {
                // Pop the page at the end of the store:
                self.page_store.pop_back();

                let page = current_source.unwrap().render_prev_page();
            }
        }
        None
    }

    pub fn get_next_page(&mut self) -> Option<&DynamicImage> {
        let mut next_page_in_store = EXTRA_PAGES_AT_ENDS;

        if self.sources.is_some() {
            let current_source = self.sources.as_mut().unwrap().get_mut(self.current_source);
            if current_source.is_some() {
                // Pop the page at the start of the store:
                self.page_store.pop_front();

                let page = current_source.unwrap().render_next_page();
                if page.is_some() {
                    // push next page from the current source to the end of the store
                    self.page_store.push_back(page.unwrap());
                } else {
                    // no pages left to render in the current source, so we try the next...
                    if self.current_source < self.source_count {
                        self.current_source += 1;

                        let current_source =
                            self.sources.as_mut().unwrap().get_mut(self.current_source);
                        if current_source.is_some() {
                            let page = current_source.unwrap().render_next_page();
                            if page.is_some() {
                                self.page_store.push_back(page.unwrap());
                            }
                            // For now I am just assuming we won't get PDFs with pages < 1
                        }
                    } else {
                        // No more pages to render
                        next_page_in_store += 1;
                    }
                }
            }

            // Get page at the middle of PageStore (current):
            return self.page_store.get(next_page_in_store);
        }
        None
    }

    pub fn get_chunk_for_page(&self, source: usize, page: usize) -> Option<Vec<DynamicImage>> {
        for i in 0..(EXTRA_PAGES_AT_ENDS * 2 + 1) {}
        None
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum SourceType {
    #[default]
    Pdf,
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

    pub fn get_pdf(&mut self) -> &pdf2image::PDF {
        &self.pdf
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
    /// Page of the source.\
    ///     source_type == PDF, then path to the PDF file;\
    ///     source_type == ImageSequence, then path to the parent dir;\
    ///     source_type == MangaDex, then url to the chapter
    path: PathBuf,
    /// Holds a PDF object if source_type == PDF
    pdf_object: Option<PDFWithDebug>,
    /// Index of the current page of the source.\
    /// This value is 0-based indexed, but both pdf2img and the front-end uses 1-based indexes.
    current_page: usize,
    page_count: usize,
}
impl Source {
    pub fn new(source_type: SourceType, path: PathBuf, keep_pdf_object: bool) -> Self {
        let mut pdf_object: Option<PDFWithDebug> = None;
        let mut page_count: usize = 0;
        match source_type {
            SourceType::Pdf => {
                let pdf = PDFWithDebug::new(&path);
                page_count = pdf.page_count();
                if keep_pdf_object {
                    pdf_object = Some(pdf);
                }
            }
            _ => {}
        }

        Self {
            source_type,
            path,
            pdf_object,
            current_page: 0,
            page_count,
        }
    }

    pub fn get_pdf_object(&mut self) {
        self.pdf_object = Some(PDFWithDebug::new(&self.path))
    }

    pub fn get_page_count(&self) -> usize {
        self.page_count
    }

    fn render_current_page(&mut self) -> Result<Vec<DynamicImage>, PDF2ImageError> {
        let pdf = self.pdf_object.as_mut().unwrap();
        pdf.get_pdf().render(
            pdf2image::Pages::Single(1 + self.current_page as u32),
            RenderOptionsBuilder::default().build()?,
        )
    }

    pub fn render_prev_page(&mut self) -> Option<DynamicImage> {
        if self.current_page > 0 {
            match self.render_current_page() {
                Ok(page_vec) => {
                    if page_vec.len() > 0 {
                        self.current_page -= 1;
                        return Some(page_vec[0].to_owned());
                    }
                }
                Err(e) => {
                    error!("Error rendering DynamicImage: {}", e);
                }
            }
        }
        None
    }

    pub fn render_next_page(&mut self) -> Option<DynamicImage> {
        if self.current_page + 1 < self.page_count {
            match self.render_current_page() {
                Ok(page_vec) => {
                    if page_vec.len() > 0 {
                        self.current_page += 1;
                        return Some(page_vec[0].to_owned());
                    }
                }
                Err(e) => {
                    error!("Error rendering DynamicImage: {}", e);
                }
            }
        }
        None
    }
}
