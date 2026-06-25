importScripts("shared-db.js");

const CAPTURE_DELAY_MS = 2500;
const DUPLICATE_WINDOW_MS = 2000;
const STEALTHMON_SYNC_URL = "http://127.0.0.1:9521/api/web-history";
const STEALTHMON_STATUS_URL = "http://127.0.0.1:9521/api/web-history/status";
const SYNC_BATCH_SIZE = 25;
const recentNavigationKeys = new Map();
let syncTimer = null;
let syncInFlight = false;

function isLoggableUrl(url) {
  if (!url) return false;
  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch (_error) {
    return false;
  }
}

function chromeCallback(fn) {
  return new Promise((resolve, reject) => {
    fn((result) => {
      const error = chrome.runtime.lastError;
      if (error) {
        reject(new Error(error.message));
      } else {
        resolve(result);
      }
    });
  });
}

function getTab(tabId) {
  return chromeCallback((done) => chrome.tabs.get(tabId, done));
}

function shouldSkipDuplicate(tabId, url, sourceEvent) {
  const key = `${tabId}|${url}|${sourceEvent}`;
  const now = Date.now();
  const lastSeen = recentNavigationKeys.get(key);
  recentNavigationKeys.set(key, now);

  for (const [storedKey, storedAt] of recentNavigationKeys.entries()) {
    if (now - storedAt > 30000) {
      recentNavigationKeys.delete(storedKey);
    }
  }

  return Boolean(lastSeen && now - lastSeen < DUPLICATE_WINDOW_MS);
}

async function logCompletedVisit(details, sourceEvent) {
  if (details.frameId !== 0 || !isLoggableUrl(details.url)) return;
  if (shouldSkipDuplicate(details.tabId, details.url, sourceEvent)) return;

  let tab;
  try {
    tab = await getTab(details.tabId);
  } catch (_error) {
    tab = null;
  }

  const visit = await WebLogDB.addVisit({
    url: details.url,
    title: tab?.title || "",
    visitedAtMs: Math.floor(details.timeStamp || Date.now()),
    incognito: Boolean(tab?.incognito),
    tabId: details.tabId,
    windowId: details.windowId,
    sourceEvent
  });

  queueStealthMonSync();
  scheduleScreenshotCapture(visit.id, details.tabId, details.windowId, details.url);
}

function scheduleScreenshotCapture(visitId, tabId, windowId, url) {
  setTimeout(() => {
    captureScreenshotForVisit(visitId, tabId, windowId, url);
  }, CAPTURE_DELAY_MS);
}

async function captureScreenshotForVisit(visitId, tabId, windowId, expectedUrl) {
  let tab;
  try {
    tab = await getTab(tabId);
  } catch (error) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: `failed_tab_unavailable: ${error.message}`
    });
    queueStealthMonSync();
    return;
  }

  if (!tab || tab.url !== expectedUrl) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: "skipped_tab_changed_before_capture"
    });
    queueStealthMonSync();
    return;
  }

  if (!tab.active) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: "pending_tab_not_active"
    });
    queueStealthMonSync();
    return;
  }

  try {
    const dataUri = await chromeCallback((done) => {
      chrome.tabs.captureVisibleTab(windowId, { format: "jpeg", quality: 70 }, done);
    });
    await WebLogDB.updateVisit(visitId, {
      title: tab.title || "",
      screenshotDataUri: dataUri || "",
      screenshotMime: dataUri?.slice(5, dataUri.indexOf(";")) || "image/jpeg",
      screenshotCapturedAtMs: Date.now(),
      screenshotStatus: dataUri ? "captured" : "failed_empty_capture"
    });
    queueStealthMonSync();
  } catch (error) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: `failed_capture: ${error.message}`
    });
    queueStealthMonSync();
  }
}

function queueStealthMonSync(delayMs = 1000) {
  if (syncTimer) return;
  syncTimer = setTimeout(() => {
    syncTimer = null;
    syncVisitsToStealthMon();
  }, delayMs);
}

function setStealthMonStatus(status) {
  return new Promise((resolve) => {
    chrome.storage.local.set(
      {
        stealthmonSyncStatus: {
          checkedAtMs: Date.now(),
          ...status
        }
      },
      resolve
    );
  });
}

async function pingStealthMon(unsyncedCount = 0) {
  const response = await fetch(STEALTHMON_STATUS_URL, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`StealthMon status failed: HTTP ${response.status}`);
  }

  const status = await response.json();
  await setStealthMonStatus({
    connected: true,
    lastError: "",
    lastSuccessAtMs: Date.now(),
    totalVisits: status.total_visits || 0,
    latestVisitAtMs: status.latest_visit_at_ms || null,
    unsyncedCount
  });
}

async function syncVisitsToStealthMon() {
  if (syncInFlight) return;
  syncInFlight = true;

  try {
    const unsynced = await WebLogDB.listUnsyncedVisits();
    if (unsynced.length === 0) {
      await pingStealthMon(0);
      return;
    }

    for (let i = 0; i < unsynced.length; i += SYNC_BATCH_SIZE) {
      const batch = unsynced.slice(i, i + SYNC_BATCH_SIZE);
      const syncStartedAt = Date.now();
      const response = await fetch(STEALTHMON_SYNC_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ visits: batch })
      });

      if (!response.ok) {
        throw new Error(`StealthMon sync failed: HTTP ${response.status}`);
      }

      await WebLogDB.markVisitsSynced(batch.map((visit) => visit.id), syncStartedAt);
    }
    await pingStealthMon(0);
  } catch (error) {
    const unsynced = await WebLogDB.listUnsyncedVisits().catch(() => []);
    await setStealthMonStatus({
      connected: false,
      lastError: error.message,
      unsyncedCount: unsynced.length
    });
    console.warn(error.message);
  } finally {
    syncInFlight = false;
  }
}

async function retryPendingCaptureForActiveTab(tabId) {
  let tab;
  try {
    tab = await getTab(tabId);
  } catch (_error) {
    return;
  }

  if (!tab?.active || !isLoggableUrl(tab.url)) return;

  const pending = await WebLogDB.findPendingCapturesForTab(tabId, tab.url);
  for (const visit of pending) {
    await captureScreenshotForVisit(visit.id, tabId, tab.windowId, tab.url);
  }
}

chrome.webNavigation.onCompleted.addListener((details) => {
  logCompletedVisit(details, "webNavigation.onCompleted");
});

chrome.webNavigation.onHistoryStateUpdated.addListener((details) => {
  logCompletedVisit(details, "webNavigation.onHistoryStateUpdated");
});

chrome.tabs.onActivated.addListener((activeInfo) => {
  retryPendingCaptureForActiveTab(activeInfo.tabId);
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === "complete" && tab.active) {
    retryPendingCaptureForActiveTab(tabId);
  }
});

chrome.runtime.onInstalled.addListener(() => {
  queueStealthMonSync(5000);
});

chrome.runtime.onStartup.addListener(() => {
  queueStealthMonSync(5000);
});

chrome.alarms.create("stealthmon-sync", { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "stealthmon-sync") {
    queueStealthMonSync();
  }
});

queueStealthMonSync(5000);
