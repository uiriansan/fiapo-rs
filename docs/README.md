# Controller
![Controller ER](https://github.com/uiriansan/fiapo/blob/main/docs/controller_er.png)

# Database
![Database ER](https://github.com/uiriansan/fiapo/blob/main/docs/db_er.png)

# Legend
- `Source`: a readable content, like a manga, manhwa, book, etc;

# Source architecture
Internally, sources in Fiapo are composed by a 4-level hierarchy component system. In order to reduce complexity, this system is always enforced:
![Internal Source Hierarchy](https://github.com/uiriansan/fiapo/blob/main/docs/internal_source_hierarchy.png)

Where:
- an `image` file will always be classified as a Page, no matter where it is located within the file structure;
- a `PDF` file will always be classified as a Chapter, no matter where it is located within the file structure.

Fiapo will create placeholder Volumes/Chapters at import time if the file structure doesn't follow the stardard.

# Importing local files
Fiapo can open both `PDF`s and `Images`. At least one supported file must be selected. The file(s) can be opened directly into the viewer or added to the library where they can be accessed again later.

## File structure stardard
Fiapo can also import a directory with all its PDFs and images, in which case a stardard file structure convention will be used:

### For images:
![Image file structure stardard](https://github.com/uiriansan/fiapo/blob/main/docs/image_file_structure_standard.png)

### For PDFs:
![PDF file structure stardard](https://github.com/uiriansan/fiapo/blob/main/docs/pdf_file_structure_standard.png)

> [!IMPORTANT]
> Non-stardard structures are supported but may lead to wrong mapping and unexpected behavior.
