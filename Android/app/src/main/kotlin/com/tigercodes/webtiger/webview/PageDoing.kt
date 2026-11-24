// What a page is doing that would keep its tab awake (052): that it plays sound, holds the
// camera or the microphone, and was typed into. The System WebView tells an app none of it.
// The adapter knows of the camera and the microphone by having handed them over; of sound and
// of typing it hears from a script of its own in the page, which says yes or no and never
// what: not what was played, and nothing of what was typed.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.PageActivityChanged

/**
 * What the adapter knows of one page that would keep its tab awake: which of its frames are
 * playing sound, whether it was typed into, whether it was given the camera or the microphone,
 * and what the core was last told. It is of the page: a commit starts it again. It has no
 * Android in it, so its rules are tested on the JVM; [WebViewPage] forwards what it hears.
 */
internal class PageDoing {
    /** The frames that are playing something that can be heard, by the names they gave themselves. */
    private val playing = HashSet<String>()

    /** The person typed into a field of the page, in whichever frame. */
    private var typedInto = false

    /** The page was given the camera or the microphone. The WebView does not say when a page lets go of one: it is held until the page goes. */
    private var captures = false

    /** What the core was last told. A page that has just committed has none of it, as the core takes it (engine.proto). */
    private var reported = PageActivityChanged()

    /**
     * The adapter cannot listen in this WebView's pages: it is too old to be given a script, or
     * an ear for one. Nothing is then known of typing, and no page is to lose a draft because
     * the browser guessed (052 D1): every page is taken to have been typed into.
     */
    var deaf = false

    /** A frame said what it is doing, all of it as it now is. */
    fun heard(said: Said) {
        if (said.sound) playing.add(said.frame) else playing.remove(said.frame)
        // What was typed stays typed for as long as the page is the tab's (052 D1).
        typedInto = typedInto || said.typed
    }

    /** The page was given the camera or the microphone. */
    fun granted() {
        captures = true
    }

    /** Another page has committed: it is doing none of it yet. */
    fun committed() {
        playing.clear()
        typedInto = false
        captures = false
        reported = PageActivityChanged()
    }

    /** What there is to tell the core now: null when it is what the core was last told. */
    fun news(): PageActivityChanged? {
        val doing = PageActivityChanged(plays_sound = playing.isNotEmpty(), captures = captures, typed_into = typedInto || deaf)
        if (doing == reported) return null
        reported = doing
        return doing
    }

    /** What a frame said: its own name for itself, and whether it plays sound and was typed into. */
    data class Said(val frame: String, val sound: Boolean, val typed: Boolean)

    companion object {
        /** The name the ear is given in a page, for as long as it takes the script to take it away. */
        const val NAME = "webtigerDoing"

        /**
         * What a frame's script says, read: its name, a bar, 0 or 1 for sound, a bar, 0 or 1
         * for typing. Null for anything else: the ear is in the page's own world for a moment,
         * and what arrives is taken for nothing it is not.
         */
        fun parse(message: String): Said? {
            val parts = message.split('|')
            if (parts.size != 3) return null
            val (frame, sound, typed) = parts
            if (frame.isEmpty() || frame.length > 64 || !frame.all { it.isLetterOrDigit() }) return null
            if (sound !in FLAGS || typed !in FLAGS) return null
            return Said(frame, sound == "1", typed == "1")
        }

        private val FLAGS = setOf("0", "1")

        /**
         * The script, put in every page and every frame as its document starts. It takes the
         * ear for itself and takes its name out of the page, before any script of the page's
         * runs: the page cannot see it, call it, or say anything in its name. It hears that a
         * key the person pressed went into a field (an event the engine made, not one a script
         * did), and that an element of the page began or stopped playing something that can
         * be heard, and says so: each frame for itself, with all it knows as it now is.
         */
        val SCRIPT =
            """
            (function () {
                'use strict';
                var tell = window.$NAME;
                try { delete window.$NAME; } catch (ignored) {}
                if (!tell) { return; }
                var frame = String(Math.random()).slice(2) + String(Date.now());
                var typed = false, sound = false;
                function say() { tell.postMessage(frame + '|' + (sound ? 1 : 0) + '|' + (typed ? 1 : 0)); }

                // Typing: what goes into a line, a larger field, or a part of the page that
                // can be edited. A box that is ticked, a slider and a button are not typing.
                var notText = { checkbox: 1, radio: 1, range: 1, color: 1, file: 1, button: 1, submit: 1, reset: 1, image: 1, hidden: 1 };
                window.addEventListener('input', function (event) {
                    if (typed || !event.isTrusted) { return; }
                    var path = event.composedPath();
                    var on = path.length ? path[0] : event.target;
                    if (!on || on.nodeType !== 1) { return; }
                    var field = on.localName === 'textarea' || (on.localName === 'input' && !notText[on.type]) || on.isContentEditable === true;
                    if (!field) { return; }
                    typed = true;
                    say();
                }, true);

                // Sound: the players of the page that are playing, with a sound that is not
                // turned off or all the way down.
                var playing = [];
                function heard(player) { return !player.paused && !player.ended && !player.muted && player.volume > 0; }
                function mind(event) {
                    var player = event.target;
                    if (!player || player.nodeType !== 1 || (player.localName !== 'audio' && player.localName !== 'video')) { return; }
                    var at = playing.indexOf(player);
                    if (heard(player)) {
                        if (at < 0) { playing.push(player); }
                    } else if (at >= 0) {
                        playing.splice(at, 1);
                    }
                    var now = playing.length > 0;
                    if (now !== sound) { sound = now; say(); }
                }
                ['play', 'playing', 'pause', 'ended', 'volumechange', 'emptied'].forEach(function (name) {
                    window.addEventListener(name, mind, true);
                });
                // A frame that goes takes its sound with it.
                window.addEventListener('pagehide', function () {
                    if (sound) { playing = []; sound = false; say(); }
                }, true);
            }());
            """.trimIndent()
    }
}
