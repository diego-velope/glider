/**
 * Shared namespace for modular TV PAL (Glider).
 * Load first, before detect-platform.js and platform modules.
 */
(function (global) {
    'use strict';

    var PLATFORM_IDS = {
        TIZEN: 'tizen',
        WEBOS: 'webos',
        VIZIO: 'vizio',
        FIRETV: 'firetv',
        ANDROID_TV: 'android_tv',
        BROWSER: 'browser'
    };

    global.GliderPAL = global.GliderPAL || {};
    global.GliderPAL.PLATFORM_IDS = PLATFORM_IDS;
    global.GliderPAL.platforms = global.GliderPAL.platforms || {};

    global.GliderPAL.registerPlatform = function (id, impl) {
        if (!impl || !impl.keyMapping) {
            console.warn('[GliderPAL] registerPlatform: missing keyMapping for', id);
            return;
        }
        global.GliderPAL.platforms[id] = impl;
    };
})(typeof window !== 'undefined' ? window : this);
