// A colour with a light and a dark value, as Design/tokens.json writes them.
public import SwiftUI
#if canImport(AppKit)
    import AppKit
#elseif canImport(UIKit)
    import UIKit
#endif

public extension Color {
    /// A colour from two 0xRRGGBBAA values, the second for dark appearances.
    init(light: UInt32, dark: UInt32) {
        #if canImport(AppKit)
            self.init(nsColor: NSColor(name: nil) { appearance in
                let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
                return NSColor(rgba: isDark ? dark : light)
            })
        #elseif canImport(UIKit)
            self.init(uiColor: UIColor { traits in
                UIColor(rgba: traits.userInterfaceStyle == .dark ? dark : light)
            })
        #endif
    }
}

/// A 0xRRGGBBAA value as four components from 0 to 1.
private struct Components {
    let red, green, blue, alpha: CGFloat

    init(_ rgba: UInt32) {
        red = CGFloat((rgba >> 24) & 0xFF) / 255
        green = CGFloat((rgba >> 16) & 0xFF) / 255
        blue = CGFloat((rgba >> 8) & 0xFF) / 255
        alpha = CGFloat(rgba & 0xFF) / 255
    }
}

#if canImport(AppKit)
    fileprivate extension NSColor {
        convenience init(rgba: UInt32) {
            let parts = Components(rgba)
            self.init(srgbRed: parts.red, green: parts.green, blue: parts.blue, alpha: parts.alpha)
        }
    }

#elseif canImport(UIKit)
    fileprivate extension UIColor {
        convenience init(rgba: UInt32) {
            let parts = Components(rgba)
            self.init(red: parts.red, green: parts.green, blue: parts.blue, alpha: parts.alpha)
        }
    }
#endif
