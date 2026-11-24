// What the adapter knows of a page that would keep its tab awake (052), with no WebView: what
// its frames say, what it is told to the core, and when.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.PageActivityChanged
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PageDoingTest {
    private fun said(frame: String, sound: Boolean = false, typed: Boolean = false) = PageDoing.Said(frame, sound, typed)

    @Test
    fun aPageThatDoesNothingHasNothingToTell() {
        val doing = PageDoing()
        assertNull(doing.news())
        doing.heard(said("a"))
        assertNull("a frame that says it does nothing changes nothing", doing.news())
    }

    @Test
    fun whatIsToldIsAllThatIsKnownAndOnlyWhenItChanges() {
        val doing = PageDoing()
        doing.heard(said("a", sound = true))
        assertEquals(PageActivityChanged(plays_sound = true), doing.news())
        assertNull("said once", doing.news())
        // Typed into as well: all of it, as it now is.
        doing.heard(said("a", sound = true, typed = true))
        assertEquals(PageActivityChanged(plays_sound = true, typed_into = true), doing.news())
        // The sound stops: what was typed is typed still, also when the frame says no more of it.
        doing.heard(said("a"))
        assertEquals(PageActivityChanged(typed_into = true), doing.news())
        assertNull(doing.news())
    }

    @Test
    fun oneFramesSoundIsThePagesAndTheLastToStopEndsIt() {
        val doing = PageDoing()
        doing.heard(said("a", sound = true))
        doing.heard(said("b", sound = true))
        assertEquals(PageActivityChanged(plays_sound = true), doing.news())
        doing.heard(said("a"))
        assertNull("the other frame plays on", doing.news())
        doing.heard(said("b"))
        assertEquals(PageActivityChanged(), doing.news())
    }

    @Test
    fun aPageGivenTheCameraOrTheMicrophoneHoldsItUntilItGoes() {
        val doing = PageDoing()
        doing.granted()
        assertEquals(PageActivityChanged(captures = true), doing.news())
        doing.heard(said("a", typed = true))
        assertEquals(PageActivityChanged(captures = true, typed_into = true), doing.news())
        // Another page: none of it, and nothing to tell until something is so.
        doing.committed()
        assertNull(doing.news())
        doing.heard(said("a", typed = true))
        assertEquals("the page that follows is typed into in its turn", PageActivityChanged(typed_into = true), doing.news())
    }

    @Test
    fun anAdapterThatCannotListenTakesEveryPageToHaveBeenTypedInto() {
        val doing = PageDoing()
        doing.deaf = true
        assertEquals(PageActivityChanged(typed_into = true), doing.news())
        assertNull(doing.news())
        doing.committed()
        assertEquals("and the next page, and every one after", PageActivityChanged(typed_into = true), doing.news())
    }

    @Test
    fun whatAFrameSaysIsReadAndAnythingElseIsNothing() {
        assertEquals(said("4821657382911", sound = true), PageDoing.parse("4821657382911|1|0"))
        assertEquals(said("7", typed = true), PageDoing.parse("7|0|1"))
        assertEquals(said("a1"), PageDoing.parse("a1|0|0"))
        for (other in listOf("", "|1|0", "frame", "frame|1", "frame|1|0|0", "frame|2|0", "frame|yes|no", "fra me|1|0", "frame|1|", "x".repeat(65) + "|1|0", "{\"sound\":true}")) {
            assertNull(other, PageDoing.parse(other))
        }
    }
}
