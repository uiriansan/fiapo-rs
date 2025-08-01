use image::DynamicImage;
use log::error;
use pdf2image::RenderOptionsBuilder;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::path::PathBuf;

/// How many extra pages will be store at both ends of the current page\
/// For example, a value of '2' means that the page store will hold up to 5 pages: previous 2, current and next 2.
const EXTRA_PAGES_AT_ENDS: usize = 2;

#[derive(Default)]
pub struct Server {
    sources: Option<Vec<Source>>,
    source_count: usize,
    /// Index of the current source
    current_source: usize,
    page_count: usize,
    /// Holds a chunk of DynamicImages to be used by the reader.\
    /// A chunk is 5 pages by default (previous two, current and next two).
    page_store: VecDeque<DynamicImage>,
    current_page_in_store: usize,
}
/// Custom debug implementation that ignores 'page_store' so it doesn't print a wall of bytes
impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("sources", &self.sources)
            .field("source_count", &self.source_count)
            .field("current_source", &self.current_source)
            .field("page_count", &self.page_count)
            .field("page_store", &self.page_store.len())
            .field("current_page_in_store", &self.current_page_in_store)
            .finish()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            sources: None,
            source_count: 0,
            current_source: 0,
            page_count: 0,
            page_store: VecDeque::with_capacity(EXTRA_PAGES_AT_ENDS * 2 + 1),
            current_page_in_store: 0,
        }
    }

    pub fn set_sources(&mut self, sources: Vec<Source>, page_count: usize) {
        self.source_count = sources.len();
        self.sources = Some(sources);
        self.current_source = 0;
        self.page_count = page_count;
        self.page_store.clear();

        // Render first chunk of pages
        self.render_chunk_for_page(0, 0);
    }

    // TODO: revise this
    pub fn get_prev_page(&mut self) -> Option<&DynamicImage> {
        let current_page = self.current_page_in_store;

        if current_page == 2 && self.page_store.len() == 3 {
            self.current_page_in_store -= 1;
        }

        self.page_store.get(current_page)
    }

    // TODO: revise this
    pub fn get_next_page(&mut self) -> Option<&DynamicImage> {
        let current_page = self.current_page_in_store;

        if current_page < 2 && self.page_store.len() > current_page + 1 {
            self.current_page_in_store += 1;
        }

        println!("{}", current_page);
        self.page_store.get(current_page)
    }

    // TODO: revise this
    fn _render_prev_page(&mut self) {
        if self.sources.is_some() {
            let current_source = self.sources.as_mut().unwrap().get_mut(self.current_source);
            if current_source.is_some() {
                // Pop the page at the end of the store:
                self.page_store.pop_back();

                let _page = current_source.unwrap()._render_prev_page();
            }
        }
    }

    // TODO: revise this
    fn _render_next_page(&mut self) {
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
                            // We are not loading PDFs with pages < 1
                        }
                    }
                    // else {
                    //     // No more pages to render
                    //     self.current_page_in_store += 1;
                    // }
                }
            }
        }
    }

    // TODO: revise this
    // TODO: TEST: create and store the page store in the Reader and keep only a mutabe reference to it in the server... Or not, cause the state of the reader won't be saved, probably
    fn render_chunk_for_page(&mut self, source: usize, page: usize) {
        // First chunk
        if source == 0 && page == 0 {
            for _i in 0..(EXTRA_PAGES_AT_ENDS + 1) {
                let current_source = self.sources.as_mut().unwrap().get_mut(self.current_source);
                if current_source.is_some() {
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
                                // We are not loading PDFs with pages < 1
                            }
                        }
                    }
                }
            }
        }
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
    _source_type: SourceType,
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
impl Ord for Source {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.file_name().cmp(&other.path.file_name())
    }
}
impl PartialOrd for Source {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        self.path.file_name() == other.path.file_name()
    }
}
impl Eq for Source {}
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
            _source_type: source_type,
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

    fn render_current_page(&mut self) -> Option<DynamicImage> {
        if self.pdf_object.is_none() {
            self.get_pdf_object();
        }
        if let Some(pdf) = self.pdf_object.as_mut() {
            match pdf.get_pdf().render(
                pdf2image::Pages::Single(1 + self.current_page as u32),
                RenderOptionsBuilder::default().build().ok()?,
            ) {
                Ok(vec_img) => {
                    if !vec_img.is_empty() {
                        return Some(vec_img[0].to_owned());
                    }
                }
                Err(e) => {
                    error!("Failed to render DynamicImage: {}", e);
                }
            }
        }
        None
    }

    pub fn _render_prev_page(&mut self) -> Option<DynamicImage> {
        if self.current_page > 0 {
            match self.render_current_page() {
                Some(page) => {
                    self.current_page -= 1;
                    return Some(page);
                }
                None => {
                    error!("Error rendering DynamicImage");
                }
            }
        } else {
            // Free PDF object once it's not needed anymore
            self.pdf_object = None;
        }
        None
    }

    pub fn render_next_page(&mut self) -> Option<DynamicImage> {
        if self.current_page + 1 < self.page_count {
            match self.render_current_page() {
                Some(page) => {
                    self.current_page += 1;
                    return Some(page);
                }
                None => {
                    error!("Error rendering DynamicImage");
                }
            }
        } else {
            // Free PDF object once it's not needed anymore
            self.pdf_object = None;
        }
        None
    }
}
