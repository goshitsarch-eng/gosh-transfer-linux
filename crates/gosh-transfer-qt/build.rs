fn main() {
    cxx_qt_build::CxxQtBuilder::new()
        .file("src/qt/bridge.rs")
        .qt_module("Gui")
        .qt_module("Widgets")
        .cc_builder(|cc| {
            cc.file("src/qt/ui.cpp");
        })
        .build();
}
