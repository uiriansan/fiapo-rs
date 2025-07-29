> [!WARNING]
> I am a newbie (Rust) dev. This project is an attempt to learn the language.\
> Redesign count: `3`\
> Refactor count: `5`

### Dependencies:
- [`Gtk4 ≥ 4.10`](https://www.gtk.org/);
- [`poppler`](https://poppler.freedesktop.org/);
- [`sqlite3`](https://www.sqlite.org/)

### Build:
```bash
cargo run --release
```

### TODO:
- [ ] \(∞) Fight Rust compiler in the `Rc<RefCell>` hell;
- [X] Render PDF file with GTK;
- [X] Load PDF files;
- [X] Image library integration;
- [X] Load files recursively;
- [ ] Load image sequences;
- [X] MangaDex integration;
- [ ] Fetch chapters from MangaDex;
- [ ] Cache MangaDex data;
- [ ] Download chapters from MangaDex;
- [ ] AniList API integration;
- [ ] OCR support for in-app translation;
- [ ] Optimize;

-- --
This project is heavily inspired by [Kotatsu](https://kotatsu.app/).
