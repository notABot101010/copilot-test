use qmetaobject::prelude::*;

fn main() {
    qml_register_type::<ChatApp>(cstr::cstr!("ChatApp"), 1, 0, cstr::cstr!("ChatApp"));
    
    let mut engine = QmlEngine::new();
    engine.load_file("qml/main.qml".into());
    engine.exec();
}

#[derive(Default, QObject)]
struct ChatApp {
    base: qt_base_class!(trait QObject),
    // We'll add properties and methods here
}
