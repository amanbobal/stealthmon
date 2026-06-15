(async function initPopup() {
  "use strict";

  // Check incognito access status
  const incognitoIndicator = document.querySelector("#incognitoIndicator");
  chrome.extension.isAllowedIncognitoAccess((isAllowedIncognito) => {
    if (isAllowedIncognito) {
      incognitoIndicator.classList.add("enabled");
      incognitoIndicator.textContent = "✓";
      incognitoIndicator.title = "Incognito mode access: Enabled";
    } else {
      incognitoIndicator.classList.add("disabled");
      incognitoIndicator.textContent = "⚠";
      incognitoIndicator.title =
        "Incognito mode access: Disabled - Click to enable";
      incognitoIndicator.style.cursor = "pointer";
      incognitoIndicator.addEventListener("click", () => {
        chrome.tabs.create({
          url: "chrome://extensions/?id=" + chrome.runtime.id,
        });
      });
    }
  });

  const summary = document.querySelector("#summary");
  const visits = await WebLogDB.listVisits();
  const captured = visits.filter(
    (visit) => visit.screenshotStatus === "captured",
  ).length;
  const incognito = visits.filter((visit) => visit.incognito).length;

  summary.textContent = `${visits.length} visits logged. ${captured} screenshots captured. ${incognito} incognito visits.`;

  document.querySelector("#openDashboard").addEventListener("click", () => {
    chrome.runtime.openOptionsPage();
  });

  function escapeHtml(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#039;");
  }

  function escapeMarkdownTable(value) {
    return String(value ?? "")
      .replaceAll("|", "\\|")
      .replaceAll("\n", " ");
  }

  document
    .querySelector("#exportMarkdown")
    .addEventListener("click", async () => {
      const rows = await WebLogDB.listVisits();
      const header = [
        "---",
        `exportedAt: ${new Date().toISOString()}`,
        "source: vault-web-logger",
        "dedupeKey: id",
        "---",
        "",
        "| Date | Time | Mode | Website | Screenshot | ID |",
        "| --- | --- | --- | --- | --- | --- |",
      ];
      const body = rows.map((row) => {
        const screenshot = row.screenshotDataUri
          ? `<img src="${row.screenshotDataUri}" width="240" />`
          : escapeHtml(row.screenshotStatus || "missing");
        return [
          row.date,
          row.time,
          row.context,
          `[${escapeMarkdownTable(row.host || row.url)}](${row.url})`,
          screenshot,
          row.id,
        ].join(" | ");
      });
      const content = `${header.join("\n")}\n${body.map((line) => `| ${line} |`).join("\n")}\n`;
      const url = URL.createObjectURL(
        new Blob([content], { type: "text/markdown" }),
      );
      await new Promise((resolve, reject) => {
        chrome.downloads.download(
          {
            url,
            filename: `vault-web-log-all-${new Date().toISOString().slice(0, 10)}.md`,
            saveAs: true,
          },
          (downloadId) => {
            const error = chrome.runtime.lastError;
            if (error) reject(new Error(error.message));
            else resolve(downloadId);
          },
        );
      });
      setTimeout(() => URL.revokeObjectURL(url), 30000);
    });
})();
