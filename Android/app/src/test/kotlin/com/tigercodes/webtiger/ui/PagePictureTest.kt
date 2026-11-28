// A sleeping tab's picture in a window (059): the window's model against a scripted core, and
// the picture drawn by Compose on Robolectric's Android with its real pictures. When the
// window asks the core for the picture its tab names, what it shows and until when, what it
// does with what is no picture, and where the picture is drawn.
package com.tigercodes.webtiger.ui

import android.graphics.Bitmap
import android.graphics.Color
import androidx.activity.ComponentActivity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.assertContentDescriptionEquals
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.StorageVerdict
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.PagePicture
import com.tigercodes.webtiger.core.v1.PicturesPage
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.io.ByteArrayOutputStream
import kotlin.math.abs
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.test.runTest
import okio.ByteString.Companion.toByteString
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class PagePictureTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    /** What the scripted core has for each number: a picture, or bytes that are none. */
    private val kept = HashMap<Long, ByteArray>()

    private fun script(request: Request): Response {
        val asked = request.query_pictures ?: return windowScript(request)
        return Response(pictures_page = PicturesPage(pictures = asked.picture_ids.mapNotNull { number -> kept[number]?.let { PagePicture(picture_id = number, jpeg = it.toByteString()) } }))
    }

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    @After
    fun end() {
        scope.cancel()
    }

    /** A picture whose top half is one colour and whose bottom half another, as a JPEG (or, asked for one, as a PNG). */
    private fun picture(wide: Int, tall: Int, format: Bitmap.CompressFormat = Bitmap.CompressFormat.JPEG): ByteArray {
        val pixels = IntArray(wide * tall) { index -> if (index / wide < tall / 2) TOP else BOTTOM }
        val bitmap = Bitmap.createBitmap(pixels, wide, tall, Bitmap.Config.ARGB_8888)
        return ByteArrayOutputStream().also { bitmap.compress(format, 100, it) }.toByteArray()
    }

    /** The tab in front as the core says it is: on its way, and naming this picture (0: none). */
    private fun tab(picture: Long, id: Long = 2, progress: Double = 0.1): TabView =
        TabView(tab_id = id, web_view_id = 4, load_state = LoadState.LOAD_STATE_LOADING, progress = progress, picture_id = picture)

    private fun said(picture: Long, tab: Long = 2, progress: Double = 0.1): WindowSnapshot = WindowSnapshot(window_id = 1, tab = tab(picture, tab, progress))

    /** The numbers a core was asked the pictures of, question by question. */
    private fun asked(core: ScriptedCore): List<List<Long>> = core.requests.mapNotNull { it.query_pictures?.picture_ids }

    @Test
    fun theWindowAsksForThePictureItsTabNamesOnceAndShowsItUntilTheTabNamesNone() =
        runTest {
            kept[7] = picture(800, 600)
            kept[9] = picture(400, 900)
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            // A tab with no picture: nothing is asked, and nothing shown.
            core.emit(said(picture = 0), subscription = 3)
            assertEquals(emptyList<List<Long>>(), asked(core))
            assertNull(model.pagePicture)

            // The tab in front names a picture: asked for, by its number alone, and shown.
            core.emit(said(picture = 7), subscription = 3)
            assertEquals(listOf(listOf(7L)), asked(core))
            assertEquals(800 to 600, model.pagePicture?.let { it.width to it.height })
            // Told again while the page comes: not asked again, and there still.
            core.emit(said(picture = 7, progress = 0.6), subscription = 3)
            assertEquals(1, asked(core).size)
            assertNotNull(model.pagePicture)

            // The page has arrived: the tab names none, and there is none.
            core.emit(said(picture = 0, progress = 0.8), subscription = 3)
            assertNull(model.pagePicture)
            assertEquals(1, asked(core).size)

            // Another tab in front, waking with its own; and the first again, on its way
            // still: each is asked for as it is named.
            core.emit(said(picture = 9, tab = 5), subscription = 3)
            assertEquals(400 to 900, model.pagePicture?.let { it.width to it.height })
            core.emit(said(picture = 0, tab = 6), subscription = 3)
            assertNull(model.pagePicture)
            core.emit(said(picture = 9, tab = 5), subscription = 3)
            assertEquals(listOf(listOf(7L), listOf(9L), listOf(9L)), asked(core))
            assertEquals(400, model.pagePicture?.width)
        }

    @Test
    fun whatIsNoPictureIsNotDrawn() =
        runTest {
            // 1: the core has none of it. 2: words. 3: a PNG. 4: a JPEG wider than the core
            // keeps. 5: the first bytes of a JPEG and nothing after. 6: a picture.
            kept[2] = "<!doctype html>".toByteArray()
            kept[3] = picture(800, 600, Bitmap.CompressFormat.PNG)
            kept[4] = picture(PICTURE_MOST_WIDE + 1, 600)
            kept[5] = byteArrayOf(0xFF.toByte(), 0xD8.toByte(), 0xFF.toByte())
            kept[6] = picture(320, 240)
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            for (number in 1L..5L) {
                core.emit(said(picture = number), subscription = 3)
                assertEquals(number.toInt(), asked(core).size)
                assertNull("picture $number is none", model.pagePicture)
            }
            core.emit(said(picture = 6), subscription = 3)
            assertEquals("and the next that is one is shown", 320, model.pagePicture?.width)

            // What is a picture to draw: a JPEG, no wider than the core keeps, and no taller than a window.
            assertNotNull(pictureOfAPage(picture(PICTURE_MOST_WIDE, 2)))
            assertNull(pictureOfAPage(picture(PICTURE_MOST_WIDE + 1, 2)))
            assertNotNull(pictureOfAPage(picture(2, PICTURE_MOST_TALL)))
            assertNull(pictureOfAPage(picture(2, PICTURE_MOST_TALL + 1)))
            assertNull(pictureOfAPage(picture(80, 60, Bitmap.CompressFormat.PNG)))
            assertNull(pictureOfAPage(ByteArray(0)))
        }

    /** A core whose answers about pictures wait until the test lets each go, in whatever order it likes. */
    private class HeldCore : CoreClient {
        val held = ArrayList<CompletableDeferred<Response>>()
        override val events: Flow<Event> = emptyFlow()

        override suspend fun request(request: Request): Response {
            if (request.query_pictures == null) return windowScript(request)
            return CompletableDeferred<Response>().also(held::add).await()
        }

        override fun report(event: EngineEvent) = Unit

        override fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean) = NavigationVerdict.ALLOW

        override fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType) = RequestVerdict.ALLOW

        override fun storageVerdict(origin: String, topOrigin: String, pageUrl: String, webViewId: Long) = StorageVerdict.OWN

        override fun stop() = Unit
    }

    private fun answer(number: Long, jpeg: ByteArray) = Response(pictures_page = PicturesPage(pictures = listOf(PagePicture(picture_id = number, jpeg = jpeg.toByteString()))))

    @Test
    fun anAnswerThatComesAfterTheTabNamesAnotherPictureOrNoneIsNotShown() {
        val slow = HeldCore()
        val model = BrowserModel(BrowserSession(slow, SystemWebViewEngine(FakePages()), scope))
        val (small, large) = picture(320, 240) to picture(800, 600)

        // Named, and the page arrives before the picture does: the picture is nobody's.
        model.showPicture(tab(picture = 7))
        assertEquals(1, slow.held.size)
        model.showPicture(tab(picture = 0))
        slow.held[0].complete(answer(7, small))
        assertNull("the page was there first", model.pagePicture)

        // One named and then another, and the first's answer comes: not shown. The second's is.
        model.showPicture(tab(picture = 7))
        model.showPicture(tab(picture = 9, id = 5))
        assertEquals(3, slow.held.size)
        slow.held[1].complete(answer(7, small))
        assertNull("an older picture is not the one named now", model.pagePicture)
        slow.held[2].complete(answer(9, large))
        assertEquals(800, model.pagePicture?.width)

        // An answer that carries another picture than was asked for is not shown either.
        model.showPicture(tab(picture = 0))
        model.showPicture(tab(picture = 11))
        slow.held[3].complete(answer(12, small))
        assertNull(model.pagePicture)
    }

    @Test
    fun aCoreThatWillNotAnswerShowsNoPicture() =
        runTest {
            val refusing = ScriptedCore { request -> if (request.query_pictures != null) throw com.tigercodes.webtiger.core.CoreError.Refused("request.unsupported", "") else windowScript(request) }
            val model = BrowserModel(unconfinedSession(refusing))
            model.open()
            refusing.emit(said(picture = 7), subscription = 3)
            assertNull(model.pagePicture)
            assertEquals("the tab is in front all the same", 4L, model.tab.web_view_id)
        }

    /** Whether a dot of what is drawn is nearly this colour: the place given as shares of the place's width and height, from its top left. */
    private fun isNearly(colour: Int, drawn: Bitmap, across: Double, down: Double): Boolean {
        val dot = drawn.getPixel((drawn.width * across).toInt(), (drawn.height * down).toInt())
        return listOf(Color.red(dot) - Color.red(colour), Color.green(dot) - Color.green(colour), Color.blue(dot) - Color.blue(colour)).all { abs(it) < 40 }
    }

    /** The picture drawn over a green place of this size, in dp: what the place then looks like. */
    private fun drawn(picture: androidx.compose.ui.graphics.ImageBitmap, wide: Int, tall: Int): Bitmap {
        compose.setContent {
            Box(Modifier.size(wide.dp, tall.dp).background(androidx.compose.ui.graphics.Color(UNDER)).testTag("place")) { PageAsItWas(picture) }
        }
        compose.waitForIdle()
        val whole = Snapshots.picture(compose.activity.window.decorView)
        val place = compose.onNodeWithTag("place").fetchSemanticsNode().boundsInRoot
        return Bitmap.createBitmap(whole, place.left.toInt(), place.top.toInt(), place.width.toInt(), place.height.toInt())
    }

    @Test
    @Config(qualifiers = "w411dp-h891dp-xhdpi")
    fun thePictureIsDrawnFromTheTopAsWideAsItsPlaceInItsOwnShapeAndSaysWhatItIs() {
        val picture = checkNotNull(pictureOfAPage(picture(800, 600)))
        // A place of the picture's shape: the picture fills it, its top half above its bottom.
        val same = drawn(picture, 400, 300)
        assertTrue(isNearly(TOP, same, 0.5, 0.25))
        assertTrue(isNearly(BOTTOM, same, 0.5, 0.75))
        assertTrue("to its corners", isNearly(TOP, same, 0.02, 0.02) && isNearly(BOTTOM, same, 0.98, 0.98))
        compose.onNodeWithTag("page-picture").assertContentDescriptionEquals("The page as it was")
    }

    @Test
    @Config(qualifiers = "w411dp-h891dp-xhdpi")
    fun inATallerPlaceThePictureIsAsWideFromTheTopAndWhatIsUnderShowsBelowIt() {
        val picture = checkNotNull(pictureOfAPage(picture(800, 600)))
        val taller = drawn(picture, 400, 600)
        assertTrue(isNearly(TOP, taller, 0.5, 0.125))
        assertTrue(isNearly(BOTTOM, taller, 0.5, 0.375))
        assertTrue("the picture is not made taller than its shape", isNearly(UNDER, taller, 0.5, 0.75))
    }

    @Test
    @Config(qualifiers = "w411dp-h891dp-xhdpi")
    fun inAShorterPlaceThePictureIsAsWideFromTheTopAndCutOffAtThePlacesEnd() {
        val picture = checkNotNull(pictureOfAPage(picture(800, 600)))
        val shorter = drawn(picture, 400, 200)
        assertTrue(isNearly(TOP, shorter, 0.5, 0.5))
        assertTrue(isNearly(BOTTOM, shorter, 0.5, 0.9))
        assertTrue("as wide as the place", isNearly(TOP, shorter, 0.02, 0.5) && isNearly(TOP, shorter, 0.98, 0.5))
    }

    private companion object {
        val TOP = Color.rgb(0xCC, 0x1A, 0x1A)
        val BOTTOM = Color.rgb(0x1A, 0x33, 0xCC)
        val UNDER = Color.rgb(0x1A, 0xCC, 0x1A)
    }
}
