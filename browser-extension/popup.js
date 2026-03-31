/**
 * S.O.G. Battle — Popup Logic
 * Handles STT (Speech-to-Text), command matching, and UI.
 */

(function () {
  "use strict";

  // --- DOM refs ---
  const toggleBtn = document.getElementById("toggle-btn");
  const statusDot = document.getElementById("status-indicator");
  const statusText = document.getElementById("status-text");
  const commandsList = document.getElementById("commands-list");
  const logContainer = document.getElementById("log-container");

  // --- State ---
  let isActive = false;
  let recognition = null;

  // --- Init command list UI ---
  COMMANDS.forEach((cmd) => {
    const li = document.createElement("li");
    li.textContent = cmd.label;
    commandsList.appendChild(li);
  });

  // --- Text normalization ---
  function normalizeText(text) {
    return text
      .toLowerCase()
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "") // strip accents
      .replace(/\s+/g, " ")
      .trim();
  }

  // --- Command matching (longest pattern first) ---
  function matchCommand(rawTranscript) {
    const text = normalizeText(rawTranscript);

    // Sort patterns longest-first across all commands
    const allPatterns = COMMANDS.flatMap((cmd) =>
      cmd.patterns.map((p) => ({
        command: cmd,
        pattern: normalizeText(p),
        length: normalizeText(p).length,
      }))
    );

    allPatterns.sort((a, b) => b.length - a.length);

    for (const entry of allPatterns) {
      if (text.includes(entry.pattern)) {
        return entry.command;
      }
    }

    return null;
  }

  // --- Logging ---
  function addLog(message, type = "info") {
    // Remove placeholder on first log
    const placeholder = logContainer.querySelector(".log-placeholder");
    if (placeholder) placeholder.remove();

    const entry = document.createElement("div");
    entry.className = "log-entry";

    const time = new Date().toLocaleTimeString("pt-BR", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });

    if (type === "error") {
      entry.innerHTML = `<span class="log-time">${time}</span><span class="log-error">${message}</span>`;
    } else {
      entry.innerHTML = `<span class="log-time">${time}</span><span class="log-match">${message}</span>`;
    }

    logContainer.prepend(entry);

    // Keep max 50 entries
    while (logContainer.children.length > 50) {
      logContainer.lastChild.remove();
    }
  }

  // --- Audio playback via background ---
  function playAudio(audioFile) {
    chrome.runtime.sendMessage(
      { type: "PLAY_AUDIO", audioFile: audioFile },
      (response) => {
        if (chrome.runtime.lastError) {
          console.error("[SOG] Send error:", chrome.runtime.lastError.message);
          addLog("Erro ao reproduzir áudio", "error");
        }
      }
    );
  }

  // --- Speech Recognition setup ---
  function createRecognition() {
    const SpeechRecognition =
      window.SpeechRecognition || window.webkitSpeechRecognition;

    if (!SpeechRecognition) {
      addLog("Speech Recognition não suportada neste navegador", "error");
      return null;
    }

    const rec = new SpeechRecognition();
    rec.continuous = true;
    rec.interimResults = true;
    rec.lang = "pt-BR";

    rec.onresult = (event) => {
      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        if (result.isFinal) {
          const transcript = result[0].transcript;
          const matched = matchCommand(transcript);

          if (matched) {
            addLog(`"${transcript}" → ${matched.response}`);
            playAudio(matched.audioFile);
          }
        }
      }
    };

    rec.onerror = (event) => {
      if (event.error === "no-speech") return; // ignore silence
      if (event.error === "aborted") return; // expected on stop
      addLog(`STT Error: ${event.error}`, "error");
    };

    rec.onend = () => {
      // Auto-restart if still active (STT stops on its own)
      if (isActive) {
        try {
          rec.start();
        } catch (e) {
          // already started — ignore
        }
      }
    };

    return rec;
  }

  // --- Toggle ---
  function startListening() {
    recognition = createRecognition();
    if (!recognition) return;

    try {
      recognition.start();
      isActive = true;
      updateUI();
      addLog("Escutando comandos…");
    } catch (e) {
      addLog("Erro ao iniciar: " + e.message, "error");
    }
  }

  function stopListening() {
    isActive = false;
    if (recognition) {
      recognition.stop();
      recognition = null;
    }
    updateUI();
    addLog("Desativado");
  }

  function updateUI() {
    toggleBtn.textContent = isActive ? "Desativar" : "Ativar";
    toggleBtn.className = "toggle-btn " + (isActive ? "active" : "inactive");
    statusDot.className = "status-dot " + (isActive ? "on" : "off");
    statusText.textContent = isActive ? "Escutando…" : "Desativado";
  }

  toggleBtn.addEventListener("click", () => {
    if (isActive) {
      stopListening();
    } else {
      startListening();
    }
  });

  // --- Init UI ---
  updateUI();
})();
