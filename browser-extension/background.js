/**
 * S.O.G. Battle — Background Service Worker
 * Handles audio playback so it continues when the popup closes.
 */

let currentAudio = null;

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === "PLAY_AUDIO") {
    const audioFile = message.audioFile;
    const url = chrome.runtime.getURL("audio/" + audioFile);

    // Cancel previous audio if playing
    if (currentAudio) {
      currentAudio.pause();
      currentAudio.currentTime = 0;
      currentAudio = null;
    }

    currentAudio = new Audio(url);
    currentAudio.play().catch((err) => {
      console.error("[SOG] Audio playback failed:", err);
      sendResponse({ success: false, error: err.message });
      return;
    });

    sendResponse({ success: true });

    // Clean up reference when done
    currentAudio.addEventListener("ended", () => {
      currentAudio = null;
    });

    return true; // keep sendResponse channel open for async
  }
});
