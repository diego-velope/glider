/**
 * Vizio
 */
(function (global) {
    'use strict';

    var P = global.GliderPAL.PLATFORM_IDS;

    global.GliderPAL.registerPlatform(P.VIZIO, {
        keyMapping: {
            up: [38],
            down: [40],
            left: [37],
            right: [39],
            action: [13],
            back: [8]
        }
    });
})(typeof window !== 'undefined' ? window : this);
