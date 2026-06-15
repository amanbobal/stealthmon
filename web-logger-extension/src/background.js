importScripts("shared-db.js");

const CAPTURE_DELAY_MS = 2500;
const DUPLICATE_WINDOW_MS = 2000;
const recentNavigationKeys = new Map();

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
    return;
  }

  if (!tab || tab.url !== expectedUrl) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: "skipped_tab_changed_before_capture"
    });
    return;
  }

  if (!tab.active) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: "pending_tab_not_active"
    });
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
  } catch (error) {
    await WebLogDB.updateVisit(visitId, {
      screenshotStatus: `failed_capture: ${error.message}`
    });
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
