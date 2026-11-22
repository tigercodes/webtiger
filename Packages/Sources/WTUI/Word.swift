// A string of the catalogue of words (040). Every user-visible string is in
// Localization/strings.json; Core/tools/strings-gen writes the String Catalog the system looks
// strings up in, and Words.generated.swift, where each string has a name. The code asks by
// name, `Text(Words.tabsNew)`, so that a string that is deleted or renamed stops the build.
//
// Nothing is looked up differently for it: a name is the key the String Catalog is asked for.
import Foundation
import SwiftUI

/// One string of the catalogue.
struct Word: Hashable, Sendable {
    /// The key the String Catalog is asked for.
    let key: String

    /// A string by its key: what the generated names are made with. Everywhere else a string
    /// is asked for by its name, or by `init?(named:)`.
    init(key: String) {
        self.key = key
    }

    /// The string a key names, when the catalogue has one: for a key that is made while the
    /// app runs (an error's words are "error." and the core's code). Nothing for a key the
    /// catalogue does not have, so that whoever asks says what is shown instead.
    init?(named key: String) {
        guard Word.keys.contains(key) else { return nil }
        self.key = key
    }

    /// The words, in the app's language or in `locale`'s.
    func string(_ locale: Locale = .current) -> String {
        String(localized: String.LocalizationValue(key), bundle: .module, locale: locale)
    }

    /// The words with their arguments put in, in the app's language or in `locale`'s.
    func filled(_ arguments: any CVarArg..., locale: Locale = .current) -> String {
        String(format: string(locale), arguments: arguments)
    }
}

extension Text {
    /// The words of a string of the catalogue, in the language of where it is drawn.
    init(_ word: Word) {
        self.init(LocalizedStringKey(word.key), bundle: .module)
    }
}
