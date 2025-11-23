use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicI32, Ordering};

// 1. Un état global simple (Compteur)
static COUNTER: AtomicI32 = AtomicI32::new(0);

#[unsafe(no_mangle)]
pub extern "system" fn Java_aeska_kistaverk_MainActivity_dispatch(
    mut env: JNIEnv,     // Remis en 'mut' car on va l'utiliser pour créer la string de retour
    _class: JClass,
    input: JString,
) -> jstring {
    
    // 2. Lire l'action envoyée par Kotlin
    // On convertit l'input Java en String Rust
    let input_str: String = env.get_string(&input)
        .map(|s| s.into())
        .unwrap_or_else(|_| "{}".to_string());

    // On essaie de parser le JSON (ex: { "action": "increment" })
    let input_json: Value = serde_json::from_str(&input_str).unwrap_or(json!({}));
    let action = input_json["action"].as_str().unwrap_or("init");

    // 3. Logique Métier (Le Switch)
    match action {
        "increment" => {
            COUNTER.fetch_add(1, Ordering::Relaxed);
        },
        "reset" => {
            COUNTER.store(0, Ordering::Relaxed);
        },
        _ => {} // "init" ou autre : on ne fait rien
    }

    // 4. Récupérer la valeur actuelle
    let count = COUNTER.load(Ordering::Relaxed);

    // 5. Construire l'UI dynamique
    let screen_json = json!({
        "type": "Column",
        "padding": 50,
        "children": [
            {
                "type": "Text",
                "text": format!("Compteur Rust : {}", count), // Texte dynamique !
                "size": 30.0
            },
            {
                "type": "Button",
                "text": "Incrémenter (+1)",
                "action": "increment" // L'action à renvoyer au prochain clic
            },
            {
                "type": "Button",
                "text": "Remise à zéro",
                "action": "reset"
            }
        ]
    });

    let output = env.new_string(screen_json.to_string())
        .expect("Couldn't create java string!");

    output.into_raw()
}