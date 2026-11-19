// The window's tabs in a row (005 §5), with the buttons that stand beside them.
import SwiftUI
import WTContract
import WTDesign

/// The window's tabs in a row: each by its name, the one in front marked, a button to close each
/// and one to open another (005 §5).
struct TabStrip: View {
    let model: BrowserModel
    /// Opens the find bar (010), the zoom bar (012) and the History list (008): their buttons stand at the end of
    /// the row, beside the tabs and not among them, where they stay when the tabs scroll.
    var onFind: () -> Void = {}
    var onZoom: () -> Void = {}
    var onHistory: () -> Void = {}
    /// How wide a tab's name may be. It grows with the text's size, so that at the largest
    /// sizes a short name is still whole; a long title is cut to one line at any size.
    @ScaledMetric private var widestName: CGFloat = 180

    var body: some View {
        HStack(spacing: 0) {
            tabs
            Button(action: onFind) {
                Label { Text("find.open", bundle: .module) } icon: { Image(systemName: "magnifyingglass") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            // There is nothing to find in a tab with no page.
            .disabled(!model.hasPage)
            .accessibilityIdentifier("find")
            Button(action: onZoom) {
                Label { Text("zoom.open", bundle: .module) } icon: { Image(systemName: "textformat.size") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            // There is nothing to zoom without a page that can be.
            .disabled(model.zoom == nil)
            .accessibilityIdentifier("zoom")
            Button(action: onHistory) {
                Label { Text("history.open", bundle: .module) } icon: { Image(systemName: "clock.arrow.circlepath") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("history")
            .padding(.trailing, Tokens.Space.m)
        }
    }

    private var tabs: some View {
        ScrollViewReader { row in
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: Tokens.Space.xs) {
                    ForEach(model.tabs, id: \.tabID) { tab in
                        chip(tab).id(tab.tabID)
                    }
                    Button {
                        model.openTab()
                    } label: {
                        Label { Text("tabs.new", bundle: .module) } icon: { Image(systemName: "plus") }
                            .labelStyle(.iconOnly)
                            .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                            .contentShape(Rectangle())
                    }
                    .buttonStyle(.borderless)
                    .accessibilityIdentifier("new-tab")
                }
                .padding(.horizontal, Tokens.Space.m)
            }
            // The tab that comes in front is brought into view.
            .onChange(of: model.tab.tabID) { _, front in
                row.scrollTo(front)
            }
        }
        .frame(minHeight: BrowserWindowView.hitSize + Tokens.Space.s)
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text("tabs.list", bundle: .module))
        .accessibilityIdentifier("tab-strip")
    }

    private func chip(_ tab: WTTabSummary) -> some View {
        let isFront = tab.tabID == model.tab.tabID
        return HStack(spacing: 0) {
            Button {
                model.activate(tab.tabID)
            } label: {
                HStack(spacing: Tokens.Space.xs) {
                    if tab.loading {
                        // A dot, not a spinner: a tab that is loading is marked, and stays still.
                        Circle()
                            .fill(Tokens.Palette.accent)
                            .frame(width: 6, height: 6)
                            .accessibilityHidden(true)
                    }
                    name(tab)
                        .lineLimit(1)
                        .fontWeight(isFront ? .semibold : .regular)
                }
                .padding(.leading, Tokens.Space.s)
                .frame(maxWidth: widestName, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .fixedSize(horizontal: true, vertical: false)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("tab")
            .accessibilityAddTraits(isFront ? .isSelected : [])
            Button {
                model.closeTab(tab.tabID)
            } label: {
                Label { Text("tabs.close", bundle: .module) } icon: { Image(systemName: "xmark") }
                    .labelStyle(.iconOnly)
                    .font(.caption)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("close-tab")
        }
        .background(isFront ? Tokens.Palette.fieldFill : .clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
    }

    /// A tab's name: its page's title, or its site, or the words for a tab with no page.
    private func name(_ tab: WTTabSummary) -> Text {
        if !tab.title.isEmpty {
            Text(verbatim: tab.title)
        } else if !tab.host.isEmpty {
            Text(verbatim: tab.host)
        } else {
            Text("tabs.untitled", bundle: .module)
        }
    }
}
