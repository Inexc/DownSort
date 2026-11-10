fn main() {
    if std::env::var("TARGET").map_or(false, |t| t.contains("windows")) {
        let _ = embed_resource::compile("app.rc", embed_resource::NONE);
    }
}
