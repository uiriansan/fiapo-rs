> [!WARNING]
> I am a newbie (Rust) dev. This project is an attempt to learn the language.
> Redesign count: `3`
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
- [ ] Built-in file/chapter manager;
- [ ] Load files recursively;
- [ ] Plug-in backend support;
- [ ] MangaDex integration;
- [ ] Fetch chapters from MangaDex;
- [ ] AniList API integration;
- [ ] Optimize;
- [ ] Download chapters from MangaDex;
- [ ] OCR support for in-app translation;

-- --
This project is heavily inspired by [Kotatsu](https://kotatsu.app/).
