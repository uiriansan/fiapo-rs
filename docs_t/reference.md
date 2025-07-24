# Controller
![Controller ER](https://github.com/uiriansan/fiapo/blob/main/docs_t/controller_er.png)

# Database
![Database ER](https://github.com/uiriansan/fiapo/blob/main/docs_t/db_er.png)

# Source architecture
Internally, Fiapo has a 4-level hierarchy system that is always enforced:
![Internal Source Hierarchy](https://github.com/uiriansan/fiapo/blob/main/docs_t/internal_source_hierarchy.png)


Where:

- an `image` file will always be classified as a Page, no matter where it is located within the file structure;
- a `PDF` file will always be classified as a Chapter, no matter where it is located within the file structure.
