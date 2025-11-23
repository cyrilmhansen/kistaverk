package aeska.kistaverk

import android.os.Bundle
import android.view.ViewGroup
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // 1. Initialisation : On demande l'écran de départ
        refreshUi("init")
    }

    // Fonction qui gère le cycle : Action -> Rust -> Nouvel Écran
    private fun refreshUi(action: String) {
        // A. Créer la commande JSON pour Rust
        // Astuce simple pour faire du JSON sans librairie externe en Kotlin
        val commandJson = "{ \"action\": \"$action\" }"

        // B. Envoyer à Rust et récupérer le nouveau JSON d'UI
        val newUiJson = dispatch(commandJson)

        // C. Instancier le Renderer avec le callback
        val renderer = UiRenderer(this) { actionClicked ->
            // D. Quand on clique, on recommence la boucle !
            refreshUi(actionClicked)
        }

        // E. Convertir le JSON en Vues Android
        val rootView = renderer.render(newUiJson)

        // F. Afficher (Remplacer l'écran actuel)
        setContentView(rootView)
    }

    external fun dispatch(input: String): String

    companion object {
        init {
            System.loadLibrary("kistaverk_core")
        }
    }
}