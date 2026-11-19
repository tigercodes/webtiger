// The window's model and the places offered as its address field is typed in (011), against a
// scripted core: what it asks, what it shows, and when the list goes.
package com.tigercodes.webtiger.ui

import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.AddressSuggestions
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

/**
 * A core that opens window 1 as [windowScript] does, and offers three places named after what
 * was typed.
 */
internal fun offeringScript(request: Request): Response {
    val ask = request.suggest_addresses ?: return windowScript(request)
    return Response(
        address_suggestions =
            AddressSuggestions(
                suggestions =
                    (1..3).map { number ->
                        AddressSuggestion(
                            url = "https://example.com/${ask.text}/$number",
                            title = "${ask.text} $number",
                            display = "example.com/${ask.text}/$number",
                        )
                    },
            ),
    )
}

/** A core whose answer to what to offer for `slow` waits until it is let go: an answer that comes late. */
private class LateCore(private val core: ScriptedCore, private val slow: String) : CoreClient by core {
    val letGo = CompletableDeferred<Unit>()

    override suspend fun request(request: Request): Response {
        val answer = core.request(request)
        if (request.suggest_addresses?.text == slow) letGo.await()
        return answer
    }
}

class SuggestionModelTest {
    private suspend fun TestScope.offering(core: ScriptedCore): BrowserModel = BrowserModel(unconfinedSession(core)).also { it.open() }

    private fun asked(core: ScriptedCore): List<String> = core.requests.mapNotNull { it.suggest_addresses }.map { "${it.window_id} ${it.text}" }

    /** A snapshot whose tab in front is `front`, with `others` beside it in the strip. */
    private fun tabs(
        front: Long,
        url: String,
        display: String,
        others: List<Long> = emptyList(),
        security: AddressView.Security = AddressView.Security.SECURITY_UNSPECIFIED,
    ): WindowSnapshot =
        snapshot(url = url, display = display) { it.copy(tab_id = front, address = it.address?.copy(security = security)) }
            .copy(tabs = (listOf(front) + others).map { TabSummary(tab_id = it) })

    @Test
    fun typingInTheAddressFieldAsksWhatToOfferAndShowsTheAnswer() =
        runTest {
            val core = ScriptedCore(::offeringScript)
            val model = offering(core)
            core.emit(tabs(front = 2, url = "https://example.com/", display = "example.com"), subscription = 3)
            model.isEditingAddress = true
            assertEquals("what the model itself puts in the field is not typed: nothing is asked", emptyList<String>(), asked(core))

            model.addressText = "r"
            model.addressText = "ru"
            model.addressText = "rus"

            assertEquals("each letter, for the window", listOf("1 r", "1 ru", "1 rus"), asked(core))
            assertEquals("the answer to what is now in the field", listOf("rus 1", "rus 2", "rus 3"), model.suggestions.map { it.title })
        }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun anAnswerThatComesLateIsForNothing() =
        runTest {
            val scripted = ScriptedCore(::offeringScript)
            val core = LateCore(scripted, slow = "r")
            val session = BrowserSession(core, SystemWebViewEngine(FakePages()), CoroutineScope(backgroundScope.coroutineContext + UnconfinedTestDispatcher(testScheduler)))
            val model = BrowserModel(session).also { it.open() }
            model.isEditingAddress = true

            // The answer for "r" is on its way while "ru" is asked and answered.
            model.addressText = "r"
            model.addressText = "ru"
            assertEquals(listOf("ru 1", "ru 2", "ru 3"), model.suggestions.map { it.title })
            core.letGo.complete(Unit)
            assertEquals("the field holds \"ru\": what is offered for \"r\" is out of date", listOf("ru 1", "ru 2", "ru 3"), model.suggestions.map { it.title })

            // The same of an answer that arrives after the list was put away.
            val later = LateCore(scripted, slow = "late")
            val other = BrowserModel(BrowserSession(later, SystemWebViewEngine(FakePages()), session.scope)).also { it.open() }
            other.isEditingAddress = true
            other.addressText = "late"
            other.putSuggestionsAway()
            later.letGo.complete(Unit)
            assertEquals(emptyList<AddressSuggestion>(), other.suggestions)
        }

    @Test
    fun theListGoesWhenTheFieldIsEmptiedOrLosesTheKeyboard() =
        runTest {
            val core = ScriptedCore(::offeringScript)
            val model = offering(core)
            model.isEditingAddress = true
            model.addressText = "rus"
            assertEquals(3, model.suggestions.size)

            model.addressText = ""
            assertEquals("nothing typed, nothing offered", emptyList<AddressSuggestion>(), model.suggestions)
            assertEquals("and nothing is asked for nothing", 1, asked(core).size)

            model.addressText = "rus"
            assertEquals(3, model.suggestions.size)
            model.isEditingAddress = false
            assertEquals(emptyList<AddressSuggestion>(), model.suggestions)
        }

    @Test
    fun choosingAPlaceGoesThereAndPutsTheListAway() =
        runTest {
            val core = ScriptedCore(::offeringScript)
            val model = offering(core)
            model.isEditingAddress = true
            model.addressText = "rus"

            model.choose(model.suggestions[1])

            val submit = core.requests.last().submit_address
            assertEquals("https://example.com/rus/2", submit?.text)
            assertEquals(2L, submit?.tab_id)
            assertFalse(model.isEditingAddress)
            assertEquals(emptyList<AddressSuggestion>(), model.suggestions)
        }

    @Test
    fun anotherTabInFrontPutsTheListAway() =
        runTest {
            val core = ScriptedCore(::offeringScript)
            val model = offering(core)
            core.emit(tabs(front = 2, url = "https://example.com/", display = "example.com", others = listOf(9)), subscription = 3)
            model.isEditingAddress = true
            model.addressText = "rus"
            assertEquals(3, model.suggestions.size)

            core.emit(tabs(front = 9, url = "https://example.org/", display = "example.org", others = listOf(2)), subscription = 3)

            assertEquals("they were offered for what was typed for the other tab", emptyList<AddressSuggestion>(), model.suggestions)
            assertEquals("https://example.org/", model.addressText)
        }

    @Test
    fun thePadlockIsThePagesAndNotWhatWasTypeds() =
        runTest {
            val core = ScriptedCore(::offeringScript)
            val model = offering(core)
            core.emit(tabs(front = 2, url = "https://example.com/", display = "example.com", security = AddressView.Security.SECURITY_SECURE), subscription = 3)
            assertEquals("the field shows the page's address", AddressView.Security.SECURITY_SECURE, model.fieldSecurity)

            // With the keyboard and nothing typed, the field still shows the page's address.
            model.isEditingAddress = true
            assertEquals(AddressView.Security.SECURITY_SECURE, model.fieldSecurity)

            model.addressText = "exa"
            assertEquals("what was typed is nobody's page yet", AddressView.Security.SECURITY_UNSPECIFIED, model.fieldSecurity)

            // Given up, the field shows the page again, and its mark.
            model.isEditingAddress = false
            assertEquals("example.com", model.addressText)
            assertEquals(AddressView.Security.SECURITY_SECURE, model.fieldSecurity)
        }

    @Test
    fun whatWasTypedBeforeTheWindowsFirstSnapshotStays() =
        runTest {
            val core = ScriptedCore(::offeringScript)
            val model = offering(core)
            // Typed at once, before the core has said what the window's tab shows.
            model.isEditingAddress = true
            model.addressText = "exam"

            core.emit(tabs(front = 2, url = "", display = ""), subscription = 3)

            assertEquals("the first snapshot names the first tab: it is no other tab coming in front", "exam", model.addressText)
        }

    @Test
    fun aPictureOfTheListKeepsWhatItWasMadeWith() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val offered = listOf(AddressSuggestion(url = "https://example.com/", title = "Example Domain", display = "example.com"))
            val model = BrowserModel(unconfinedSession(core), typing = "exa", offered = offered)
            model.open()

            // What a window does to its model as it is first drawn: the core's snapshot, and
            // the field saying it has no keyboard.
            core.emit(tabs(front = 2, url = "https://example.org/", display = "example.org", security = AddressView.Security.SECURITY_SECURE), subscription = 3)
            model.isEditingAddress = false

            assertEquals("exa", model.addressText)
            assertEquals(offered, model.suggestions)
            assertEquals("no padlock beside what was typed", AddressView.Security.SECURITY_UNSPECIFIED, model.fieldSecurity)
            assertEquals("a picture asks the core nothing of what it shows", 0, core.requests.count { it.suggest_addresses != null })
        }
}
