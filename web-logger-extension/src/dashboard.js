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
  const exportButtons = [
    document.querySelector("#exportJsonl"),
    document.querySelector("#exportCsv"),
    document.querySelector("#exportMarkdown")
  ];
  const MAX_VISIBLE_ROWS = 500;
  const MAX_THUMBNAILS = 80;
  const MAX_EXPORT_CHUNK_CHARS = 8 * 1024 * 1024;

  let currentRows = [];
  let isExporting = false;

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

  function chromeDownload(options) {
    return new Promise((resolve, reject) => {
      chrome.downloads.download(options, (downloadId) => {
        const error = chrome.runtime.lastError;
        if (error) reject(new Error(error.message));
        else resolve(downloadId);
      });
    });
  }

  async function downloadText(filename, content, type, saveAs = true) {
    const url = URL.createObjectURL(new Blob([content], { type }));
    try {
      await chromeDownload({ url, filename, saveAs });
    } finally {
      setTimeout(() => URL.revokeObjectURL(url), 30000);
    }
  }

  function exportFilenameBase() {
    const from = controls.fromDate.value || "all";
    const to = controls.toDate.value || new Date().toISOString().slice(0, 10);
    const mode = controls.mode.value === "all" ? "all-modes" : controls.mode.value;
    return `vault-web-log-${from}_to_${to}-${mode}`;
  }

  function setExporting(exporting) {
    isExporting = exporting;
    for (const button of exportButtons) {
      button.disabled = exporting;
    }
  }

  function chunkSuffix(index) {
    return `.part-${String(index + 1).padStart(3, "0")}`;
  }

  async function downloadFormattedChunks(filenameBase, extension, rows, makeLine, type, prefixLines = []) {
    let chunkIndex = 0;
    let current = [...prefixLines];
    let currentLength = current.reduce((sum, line) => sum + line.length + 1, 0);

    async function flushChunk() {
      if (current.length <= prefixLines.length && chunkIndex > 0) return;
      statsEl.textContent = `Exporting part ${chunkIndex + 1}...`;
      const filename = `${filenameBase}${chunkSuffix(chunkIndex)}.${extension}`;
      await downloadText(filename, `${current.join("\n")}\n`, type, chunkIndex === 0);
      chunkIndex += 1;
      current = [...prefixLines];
      currentLength = current.reduce((sum, prefixLine) => sum + prefixLine.length + 1, 0);
      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    for (const row of rows) {
      const line = makeLine(row);
      const lineLength = line.length + 1;
      if (current.length > prefixLines.length && currentLength + lineLength > MAX_EXPORT_CHUNK_CHARS) {
        await flushChunk();
      }
      current.push(line);
      currentLength += lineLength;
    }

    if (current.length > prefixLines.length || chunkIndex === 0) {
      await flushChunk();
    }
    statsEl.textContent = `Export complete: ${currentRows.length} visits in ${chunkIndex} file${chunkIndex === 1 ? "" : "s"}.`;
  }

  async function exportRows(format) {
    if (isExporting) return;
    setExporting(true);
    const filenameBase = exportFilenameBase();

    try {
      if (format === "jsonl") {
        await downloadFormattedChunks(
          filenameBase,
          "jsonl",
          currentRows,
          (row) => JSON.stringify(row),
          "application/x-ndjson"
        );
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
        await downloadFormattedChunks(
          filenameBase,
          "csv",
          currentRows,
          (row) => columns.map((column) => csvCell(row[column])).join(","),
          "text/csv",
          [columns.join(",")]
        );
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
      await downloadFormattedChunks(filenameBase, "md", currentRows, (row) => {
        const screenshot = row.screenshotDataUri
          ? `<img src="${row.screenshotDataUri}" width="240" />`
          : escapeHtml(row.screenshotStatus || "missing");
        return `| ${[
          row.date,
          row.time,
          row.context,
          `[${escapeMarkdownTable(row.host || row.url)}](${row.url})`,
          screenshot,
          row.id
        ].join(" | ")} |`;
      }, "text/markdown", header);
    } catch (error) {
      statsEl.textContent = `Export failed: ${error.message}`;
    } finally {
      setExporting(false);
    }
  }

  function escapeMarkdownTable(value) {
    return String(value ?? "").replaceAll("|", "\\|").replaceAll("\n", " ");
  }

  function parseJsonl(text) {
    return text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => JSON.parse(line));
  }

  function parseCsv(text) {
    const rows = [];
    let row = [];
    let cell = "";
    let quoted = false;

    for (let index = 0; index < text.length; index += 1) {
      const char = text[index];
      const next = text[index + 1];

      if (quoted) {
        if (char === '"' && next === '"') {
          cell += '"';
          index += 1;
        } else if (char === '"') {
          quoted = false;
        } else {
          cell += char;
        }
        continue;
      }

      if (char === '"') {
        quoted = true;
      } else if (char === ",") {
        row.push(cell);
        cell = "";
      } else if (char === "\n") {
        row.push(cell);
        rows.push(row);
        row = [];
        cell = "";
      } else if (char !== "\r") {
        cell += char;
      }
    }

    if (cell || row.length) {
      row.push(cell);
      rows.push(row);
    }

    const [headers = [], ...dataRows] = rows;
    return dataRows
      .filter((dataRow) => dataRow.some((value) => value.trim()))
      .map((dataRow) => Object.fromEntries(headers.map((header, index) => [header, dataRow[index] || ""])));
  }

  function splitMarkdownTableRow(line) {
    const trimmed = line.trim().replace(/^\|/, "").replace(/\|$/, "");
    const cells = [];
    let cell = "";
    let escaped = false;

    for (const char of trimmed) {
      if (escaped) {
        cell += char;
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === "|") {
        cells.push(cell.trim());
        cell = "";
      } else {
        cell += char;
      }
    }
    cells.push(cell.trim());
    return cells;
  }

  function decodeHtml(value) {
    const textarea = document.createElement("textarea");
    textarea.innerHTML = value;
    return textarea.value;
  }

  function parseMarkdown(text) {
    return text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.startsWith("|") && !/^\|\s*-+/.test(line) && !/^\|\s*Date\s*\|/i.test(line))
      .map((line) => {
        const [date, time, context, website, screenshot, id] = splitMarkdownTableRow(line);
        const linkMatch = website.match(/\]\(([^)]+)\)/);
        const labelMatch = website.match(/^\[([^\]]*)\]/);
        const imageMatch = screenshot.match(/<img\s+[^>]*src=["']([^"']+)["']/i);

        return {
          id,
          date,
          time,
          dateTime: date && time ? `${date} ${time}` : "",
          context,
          incognito: String(context).toLowerCase() === "incognito",
          url: linkMatch ? linkMatch[1] : "",
          host: labelMatch ? labelMatch[1] : "",
          title: labelMatch ? labelMatch[1] : "",
          screenshotDataUri: imageMatch ? imageMatch[1] : "",
          screenshotStatus: imageMatch ? "captured" : decodeHtml(screenshot || "missing")
        };
      });
  }

  function parseImportFile(file, text) {
    const name = file.name.toLowerCase();
    if (name.endsWith(".jsonl") || name.endsWith(".ndjson")) {
      return parseJsonl(text);
    }
    if (name.endsWith(".csv")) {
      return parseCsv(text);
    }
    if (name.endsWith(".md") || name.endsWith(".markdown")) {
      return parseMarkdown(text);
    }

    const trimmed = text.trim();
    if (trimmed.startsWith("{")) return parseJsonl(text);
    if (trimmed.startsWith("|")) return parseMarkdown(text);
    return parseCsv(text);
  }

  async function importFile(file) {
    if (!file) return;

    try {
      statsEl.textContent = `Importing ${file.name}...`;
      const text = await file.text();
      const records = parseImportFile(file, text);
      if (!records.length) {
        statsEl.textContent = "Import skipped: no records found.";
        return;
      }

      const result = await WebLogDB.importVisits(records);
      await refresh();
      const issueCount = result.skipped + result.failed;
      const issueText = issueCount ? ` ${issueCount} skipped or failed.` : "";
      statsEl.textContent = `Import complete: ${result.added} added, ${result.updated} updated.${issueText}`;
      if (result.errors.length) {
        console.warn("Web Logger import issues", result.errors);
      }
    } catch (error) {
      statsEl.textContent = `Import failed: ${error.message}`;
    }
  }

  function renderRows(rows) {
    rowsEl.textContent = "";
    const fragment = document.createDocumentFragment();

    rows.slice(0, MAX_VISIBLE_ROWS).forEach((row, index) => {
      const tr = document.createElement("tr");
      const screenshotCell = row.screenshotDataUri && index < MAX_THUMBNAILS
        ? `<img class="thumb" src="${row.screenshotDataUri}" alt="" loading="lazy">`
        : `<span class="status">${escapeHtml(row.screenshotStatus || (row.screenshotDataUri ? "captured" : ""))}</span>`;

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
    });

    if (rows.length > MAX_VISIBLE_ROWS) {
      const tr = document.createElement("tr");
      tr.innerHTML = `<td colspan="6" class="status">${escapeHtml(rows.length - MAX_VISIBLE_ROWS)} more matching visits are hidden from the table but included in exports.</td>`;
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
    const visibleText = currentRows.length > MAX_VISIBLE_ROWS ? ` Showing ${MAX_VISIBLE_ROWS}.` : "";
    statsEl.textContent = `${currentRows.length} matching visits. ${captured} screenshots captured.${visibleText}`;
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
  document.querySelector("#importData").addEventListener("click", () => {
    document.querySelector("#importFile").click();
  });
  document.querySelector("#importFile").addEventListener("change", (event) => {
    importFile(event.target.files[0]);
    event.target.value = "";
  });

  refresh();
})();
