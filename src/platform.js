// macOS draws the window background without blur (see App.vue): what is behind the widget
// stays sharp, so the widget needs to be more opaque there to stay readable.
export const isMac = navigator.userAgent.includes("Mac");
export const DEFAULT_OPACITY = isMac ? 0.97 : 0.92;
export const MIN_OPACITY = isMac ? 0.85 : 0.2;
