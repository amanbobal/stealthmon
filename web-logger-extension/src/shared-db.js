(function attachWebLogDB(globalScope) {
  "use strict";

  const DB_NAME = "web-logger";
  const DB_VERSION = 1;
  const STORE = "visits";

  function requestToPromise(request) {
    return new Promise((resolve, reject) => {
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  function openDb() {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains(STORE)) {
          const store = db.createObjectStore(STORE, { keyPath: "id" });
          store.createIndex("visitedAtMs", "visitedAtMs", { unique: false });
          store.createIndex("host", "host", { unique: false });
          store.createIndex("incognito", "incognito", { unique: false });
          store.createIndex("url", "url", { unique: false });
        }
      };

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  async function withStore(mode, callback) {
    const db = await openDb();
    try {
      return await new Promise((resolve, reject) => {
        const tx = db.transaction(STORE, mode);
        const store = tx.objectStore(STORE);
        let callbackResult;

        tx.oncomplete = () => resolve(callbackResult);
        tx.onerror = () => reject(tx.error);
        tx.onabort = () => reject(tx.error);

        Promise.resolve(callback(store, tx))
          .then((result) => {
            callbackResult = result;
          })
          .catch((error) => {
            tx.abort();
            reject(error);
          });
      });
    } finally {
      db.close();
    }
  }

  function normalizeUrlForSearch(url) {
    try {
      const parsed = new URL(url);
      parsed.hostname = parsed.hostname.toLowerCase();
      return parsed.toString();
    } catch (_error) {
      return url;
    }
  }

  function hostFromUrl(url) {
    try {
      return new URL(url).hostname.toLowerCase();
    } catch (_error) {
      return "";
    }
  }

  function simpleDateParts(ms) {
    const date = new Date(ms);
    const pad = (value) => String(value).padStart(2, "0");
    const yyyy = date.getFullYear();
    const mm = pad(date.getMonth() + 1);
    const dd = pad(date.getDate());
    const hh = pad(date.getHours());
    const min = pad(date.getMinutes());
    const sec = pad(date.getSeconds());

    return {
      date: `${yyyy}-${mm}-${dd}`,
      time: `${hh}:${min}:${sec}`,
      dateTime: `${yyyy}-${mm}-${dd} ${hh}:${min}:${sec}`
    };
  }

  async function sha256Hex(value) {
    const bytes = new TextEncoder().encode(value);
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest))
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
  }

  async function buildVisitId({ url, visitedAtMs, incognito }) {
    const hash = await sha256Hex(`${visitedAtMs}|${incognito ? "incognito" : "normal"}|${url}`);
    return `visit_${hash.slice(0, 32)}`;
  }

  function buildSearchText(visit) {
    return [
      visit.url,
      visit.normalizedUrl,
      visit.host,
      visit.title,
      visit.context,
      visit.date,
      visit.time,
      visit.dateTime
    ]
      .filter(Boolean)
      .join(" ")
      .toLowerCase();
  }

  async function makeVisit(input) {
    const visitedAtMs = input.visitedAtMs || Date.now();
    const parts = simpleDateParts(visitedAtMs);
    const normalizedUrl = normalizeUrlForSearch(input.url);
    const incognito = Boolean(input.incognito);
    const visit = {
      id: input.id || await buildVisitId({ url: input.url, visitedAtMs, incognito }),
      url: input.url,
      normalizedUrl,
      host: input.host || hostFromUrl(input.url),
      title: input.title || "",
      visitedAtMs,
      date: parts.date,
      time: parts.time,
      dateTime: parts.dateTime,
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "",
      incognito,
      context: incognito ? "incognito" : "normal",
      screenshotDataUri: input.screenshotDataUri || "",
      screenshotMime: input.screenshotMime || "",
      screenshotCapturedAtMs: input.screenshotCapturedAtMs || null,
      screenshotStatus: input.screenshotStatus || "pending",
      tabId: input.tabId ?? null,
      windowId: input.windowId ?? null,
      sourceEvent: input.sourceEvent || "unknown",
      createdAtMs: input.createdAtMs || Date.now(),
      updatedAtMs: Date.now()
    };

    visit.searchText = buildSearchText(visit);
    return visit;
  }

  async function putVisit(visit) {
    return withStore("readwrite", async (store) => {
      await requestToPromise(store.put(visit));
      return visit;
    });
  }

  async function addVisit(input) {
    const visit = await makeVisit(input);
    return putVisit(visit);
  }

  async function updateVisit(id, patch) {
    return withStore("readwrite", async (store) => {
      const existing = await requestToPromise(store.get(id));
      if (!existing) {
        return null;
      }

      const next = {
        ...existing,
        ...patch,
        updatedAtMs: Date.now()
      };
      next.searchText = buildSearchText(next);
      await requestToPromise(store.put(next));
      return next;
    });
  }

  async function getVisit(id) {
    return withStore("readonly", (store) => requestToPromise(store.get(id)));
  }

  async function listVisits(filters = {}) {
    const visits = await withStore("readonly", (store) => requestToPromise(store.getAll()));
    const query = (filters.query || "").trim().toLowerCase();
    const mode = filters.mode || "all";
    const fromMs = filters.fromDate ? new Date(`${filters.fromDate}T00:00:00`).getTime() : null;
    const toMs = filters.toDate ? new Date(`${filters.toDate}T23:59:59.999`).getTime() : null;

    return visits
      .filter((visit) => {
        if (query && !visit.searchText.includes(query)) return false;
        if (mode === "normal" && visit.incognito) return false;
        if (mode === "incognito" && !visit.incognito) return false;
        if (fromMs !== null && visit.visitedAtMs < fromMs) return false;
        if (toMs !== null && visit.visitedAtMs > toMs) return false;
        return true;
      })
      .sort((a, b) => b.visitedAtMs - a.visitedAtMs);
  }

  async function findPendingCapturesForTab(tabId, url) {
    const visits = await listVisits();
    return visits
      .filter((visit) => {
        if (visit.tabId !== tabId) return false;
        if (visit.screenshotStatus === "captured") return false;
        if (url && visit.url !== url) return false;
        return true;
      })
      .slice(0, 5);
  }

  async function clearAllVisits() {
    return withStore("readwrite", (store) => requestToPromise(store.clear()));
  }

  globalScope.WebLogDB = {
    addVisit,
    clearAllVisits,
    findPendingCapturesForTab,
    getVisit,
    hostFromUrl,
    listVisits,
    makeVisit,
    putVisit,
    simpleDateParts,
    updateVisit
  };
})(globalThis);
