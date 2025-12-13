# i18n Extension Plan for Kistaverk

## Current State Analysis

### Existing i18n Infrastructure
- **Library**: `rust-i18n` v3
- **Current locales**: English (`en.yml`), Icelandic (`is.yml`)
- **Current translations**: Only 4 keys for home screen
- **Usage pattern**: `t!("key")` macro for translations
- **Locale switching**: Implemented in `i18n.rs` with normalization

### Current Locale Files Structure
```yaml
# en.yml
home_title: "🧰 Tool menu"
home_subtitle: "✨ Select a tool. Hash tools prompt for a file."
home_quick_access: "⚡ Quick access"
home_tools_suffix: "tools"

# is.yml
home_title: "🧰 Verkfærakistan"
home_subtitle: "✨ Veldu verkfæri. Tætiformál biðja um skrá."
home_quick_access: "⚡ Flýtiaðgangur"
home_tools_suffix: "verkfæri"
```

## Extension Plan

### Phase 1: French Locale Creation
**Priority**: High
**Files to create**: `rust/locales/fr.yml`

#### Key Areas for Translation
1. **Home Screen** (already identified)
2. **Tool Names** (Hash tools, Text tools, Image tools, etc.)
3. **UI Elements** (Buttons, labels, error messages)
4. **Feature-specific text** (Hash verification, compression, etc.)

#### French Translations Plan
```yaml
# rust/locales/fr.yml (proposed)
home_title: "🧰 Menu des outils"
home_subtitle: "✨ Sélectionnez un outil. Les outils de hachage demandent un fichier."
home_quick_access: "⚡ Accès rapide"
home_tools_suffix: "outils"

# Tool names
text_tools: "Outils de texte"
hash_tools: "Outils de hachage"
image_tools: "Outils d'image"
compression_tools: "Outils de compression"

# Common UI elements
quick_access: "Accès rapide"
search_tools: "Rechercher des outils…"
back_button: "Retour"
copy_button: "Copier"
paste_button: "Coller"
select_file: "Sélectionner un fichier"

# Hash verification
hash_verify_title: "Vérification de hachage (SHA-256)"
hash_verify_instructions: "Collez un hachage de référence, puis choisissez un fichier à vérifier."
hash_verify_copy: "Copier le hachage calculé"
hash_verify_paste: "Coller depuis le presse-papiers"
hash_verify_pick_file: "Choisir un fichier et vérifier"

# Error messages
error_prefix: "Erreur :"
file_open_failed: "Échec de l'ouverture du fichier"
read_failed: "Échec de la lecture"
write_failed: "Échec de l'écriture"

# Success messages
success_prefix: "Succès :"
result_saved: "Résultat enregistré dans : {}"
operation_complete: "Opération terminée"
```

### Phase 2: Icelandic Locale Expansion
**Priority**: Medium
**Files to update**: `rust/locales/is.yml`

#### Additional Icelandic Translations
```yaml
# Additional keys for is.yml
text_tools: "Textaverkfæri"
hash_tools: "Tætiverkfæri"
image_tools: "Myndaverkfæri"
compression_tools: "Þjappaverkfæri"

quick_access: "Flýtiaðgangur"
search_tools: "Leita að verkfærum…"
back_button: "Til baka"
copy_button: "Afrita"
paste_button: "Líma"
select_file: "Velja skrá"

hash_verify_title: "Tætistaðfesting (SHA-256)"
hash_verify_instructions: "Límaðu viðmiðunarætiskóða, veldu síðan skrá til að staðfesta."
hash_verify_copy: "Afrita reiknaða tæti"
hash_verify_paste: "Líma frá klippispjaldi"
hash_verify_pick_file: "Velja skrá og staðfesta"

error_prefix: "Villa: "
file_open_failed: "Mistókst að opna skrá"
read_failed: "Mistókst að lesa"
write_failed: "Mistókst að skrifa"

success_prefix: "Tókst: "
result_saved: "Niðurstaða vistað í: {}"
operation_complete: "Aðgerð lokin"
```

### Phase 3: System Integration
**Priority**: High

#### Required Changes
1. **Update `i18n.rs`** to support French locale:
   ```rust
   match lang {
       "is" => "is",
       "en" => "en",
       "fr" => "fr",
       _ => "en",
   }
   ```

2. **Update build system** to include new locales
3. **Test locale switching** functionality
4. **Update UI** to use i18n keys consistently

### Phase 4: Comprehensive Translation Coverage
**Priority**: Medium-Long term

#### Translation Categories
1. **All tool names and descriptions**
2. **All button labels and UI elements**
3. **All error and success messages**
4. **All help text and instructions**
5. **All feature-specific terminology**

#### Implementation Strategy
1. **Identify all hardcoded strings** in the codebase
2. **Replace with i18n keys** systematically
3. **Create comprehensive translation files**
4. **Implement fallback mechanism** for missing translations

## Technical Implementation Details

### Locale File Structure
- Files located in `rust/locales/`
- YAML format with key-value pairs
- Keys should be descriptive and consistent
- Values should include formatting placeholders where needed

### Code Changes Required
1. **Replace hardcoded strings** with `t!("key")` calls
2. **Update locale normalization** in `i18n.rs`
3. **Ensure build system** compiles all locales
4. **Add locale switching UI** if not present

### Testing Strategy
1. **Unit tests** for locale switching
2. **Integration tests** for i18n functionality
3. **UI tests** for translated content
4. **Manual testing** of all locales

## Timeline Estimate
- **Phase 1 (French locale)**: 2-3 days
- **Phase 2 (Icelandic expansion)**: 1-2 days  
- **Phase 3 (System integration)**: 1 day
- **Phase 4 (Comprehensive coverage)**: 1-2 weeks (ongoing)

## Risks and Mitigations
- **Missing translations**: Implement English fallback
- **Formatting issues**: Use consistent placeholder syntax
- **Performance impact**: Test with all locales loaded
- **UI layout issues**: Test with different language lengths

## Success Criteria
- French locale fully functional with basic translations
- Icelandic locale expanded with additional translations
- All existing functionality preserved
- No regression in performance
- Clean fallback to English for missing translations