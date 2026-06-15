(function initDashboard() {
  "use strict";

  const rowsEl = document.querySelector("#rows");
  const statsEl = document.querySelector("#stats");
  const controls = {
    query: document.querySelector("#query"),
    mode: document.querySelector("#mode"),
    fromDate: document.querySelector("#fromDate"),
    toDate: document.querySelector("#toDate")
  };

  let currentRows = [];

  function escapeHtml(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#039;");
  }

  function csvCell(value) {
    const text = String(value ?? "");
    if (/[",\r\n]/.test(text)) {
      return `"${text.replaceAll('"', '""')}"`;
    }
    return text;
  }

  function downloadText(filename, content, type) {
    const url = URL.createObjectURL(new Blob([content], { type }));
    chrome.downloads.download({ url, filename, saveAs: true }, () => {
      const error = chrome.runtime.lastError;
      if (error) {
        statsEl.textContent = `Export failed: ${error.message}`;
      }
    });
    setTimeout(() => URL.revokeObjectURL(url), 30000);
  }

  function exportFilenameBase() {
    const from = controls.fromDate.value || "all";
    const to = controls.toDate.value || new Date().toISOString().slice(0, 10);
    const mode = controls.mode.value === "all" ? "all-modes" : controls.mode.value;
    return `vault-web-log-${from}_to_${to}-${mode}`;
  }

  function exportRows(format) {
    const filenameBase = exportFilenameBase();

    if (format === "jsonl") {
      const content = currentRows.map((row) => JSON.stringify(row)).join("\n") + "\n";
      downloadText(`${filenameBase}.jsonl`, content, "application/x-ndjson");
      return;
    }

    if (format === "csv") {
      const columns = [
        "id",
        "date",
        "time",
        "dateTime",
        "context",
        "url",
        "host",
        "title",
        "screenshotDataUri",
        "screenshotStatus"
      ];
      const header = columns.join(",");
      const lines = currentRows.map((row) => columns.map((column) => csvCell(row[column])).join(","));
      downloadText(`${filenameBase}.csv`, [header, ...lines].join("\n") + "\n", "text/csv");
      return;
    }

    const header = [
      "---",
      `exportedAt: ${new Date().toISOString()}`,
      "source: vault-web-logger",
      "dedupeKey: id",
      "---",
      "",
      "| Date | Time | Mode | Website | Screenshot | ID |",
      "| --- | --- | --- | --- | --- | --- |"
    ];
    const body = currentRows.map((row) => {
      const screenshot = row.screenshotDataUri
        ? `<img src="${row.screenshotDataUri}" width="240" />`
        : escapeHtml(row.screenshotStatus || "missing");
      return [
        row.date,
        row.time,
        row.context,
        `[${escapeMarkdownTable(row.host || row.url)}](${row.url})`,
        screenshot,
        row.id
      ].join(" | ");
    });
    downloadText(`${filenameBase}.md`, `${header.join("\n")}\n${body.map((line) => `| ${line} |`).join("\n")}\n`, "text/markdown");
  }

  function escapeMarkdownTable(value) {
    return String(value ?? "").replaceAll("|", "\\|").replaceAll("\n", " ");
  }

  function renderRows(rows) {
    rowsEl.textContent = "";
    const fragment = document.createDocumentFragment();

    for (const row of rows) {
      const tr = document.createElement("tr");
      const screenshotCell = row.screenshotDataUri
        ? `<img class="thumb" src="${row.screenshotDataUri}" alt="">`
        : `<span class="status">${escapeHtml(row.screenshotStatus)}</span>`;

      tr.innerHTML = `
        <td>${escapeHtml(row.date)}</td>
        <td>${escapeHtml(row.time)}</td>
        <td><span class="mode ${row.incognito ? "is-private" : ""}">${escapeHtml(row.context)}</span></td>
        <td>
          <a href="${escapeHtml(row.url)}" target="_blank" rel="noreferrer">${escapeHtml(row.title || row.host || row.url)}</a>
          <span class="url">${escapeHtml(row.url)}</span>
        </td>
        <td>${screenshotCell}</td>
        <td><code>${escapeHtml(row.id)}</code></td>
      `;
      fragment.appendChild(tr);
    }

    rowsEl.appendChild(fragment);
  }

  async function refresh() {
    currentRows = await WebLogDB.listVisits({
      query: controls.query.value,
      mode: controls.mode.value,
      fromDate: controls.fromDate.value,
      toDate: controls.toDate.value
    });
    const captured = currentRows.filter((row) => row.screenshotStatus === "captured").length;
    statsEl.textContent = `${currentRows.length} matching visits. ${captured} screenshots captured.`;
    renderRows(currentRows);
  }

  for (const control of Object.values(controls)) {
    control.addEventListener("input", refresh);
    control.addEventListener("change", refresh);
  }

  document.querySelector("#last14Days").addEventListener("click", () => {
    const to = new Date();
    const from = new Date();
    from.setDate(to.getDate() - 13);
    controls.fromDate.value = from.toISOString().slice(0, 10);
    controls.toDate.value = to.toISOString().slice(0, 10);
    refresh();
  });

  document.querySelector("#exportJsonl").addEventListener("click", () => exportRows("jsonl"));
  document.querySelector("#exportCsv").addEventListener("click", () => exportRows("csv"));
  document.querySelector("#exportMarkdown").addEventListener("click", () => exportRows("markdown"));

  refresh();
})();
