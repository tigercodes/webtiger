// The zoom bar (012 §6): smaller, the level, bigger, and Done. Under the toolbar, where the
// find bar is when that is open instead.
import SwiftUI
import WTContract
import WTDesign

/// What it says and which of its buttons can be pressed is the core's: the level of the page
/// in front, and whether there is one above and one below.
struct ZoomBar: View {
    let model: BrowserModel

    var body: some View {
        let zoom = model.zoom ?? WTZoomView()
        HStack(spacing: Tokens.Space.s) {
            button(Words.zoomOut, symbol: "minus", identifier: "zoom-out", enabled: zoom.canZoomOut) { model.zoom(.out) }
            // The level; pressing it puts the page back to actual size.
            Button {
                model.zoom(.reset)
            } label: {
                Text(level(zoom))
                    .monospacedDigit()
                    .lineLimit(1)
                    .fixedSize()
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
            }
            // Neutral, as every button with words is (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            .disabled(zoom.percent == 100)
            .accessibilityLabel(Text(Words.zoomReset))
            .accessibilityValue(Text(level(zoom)))
            .accessibilityIdentifier("zoom-level")
            button(Words.zoomIn, symbol: "plus", identifier: "zoom-in", enabled: zoom.canZoomIn) { model.zoom(.in) }
            Spacer(minLength: 0)
            Button {
                model.closeZoom()
            } label: {
                Text(Words.zoomDone)
            }
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("zoom-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .frame(minHeight: Tokens.Size.toolbarHeight)
    }

    private func level(_ zoom: WTZoomView) -> String {
        Words.zoomLevel.filled(Int(zoom.percent))
    }

    private func button(
        _ title: Word,
        symbol: String,
        identifier: String,
        enabled: Bool,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Label { Text(title) } icon: { Image(systemName: symbol) }
                .labelStyle(.iconOnly)
                .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.borderless)
        .disabled(!enabled)
        .accessibilityIdentifier(identifier)
    }
}
