export const state = {
  pageLoadTime: Date.now(),
  firstInteractionTime: null,
  firstScrollTime: null,
  maxScrollDepth: 0,
  scrollPositions: [],
  scrollDirectionChanges: 0,
  lastScrollDirection: null,
  clickCount: 0,
  clickTimestamps: [],
  hasRageClick: false,
  hasDeadClick: false,
  mouseDistance: 0,
  lastMousePosition: null,
  keyboardEvents: 0,
  copyEvents: 0,
  visibleTime: 0,
  hiddenTime: 0,
  lastVisibilityChange: Date.now(),
  isVisible: !document.hidden,
  dataSent: false,
  tabSwitches: 0,
  focusTime: 0,
  blurCount: 0,
};

export const ENDPOINT = '/track/engagement';
export const MIN_TIME_MS = 5000;
export const RAGE_CLICK_THRESHOLD = 3;
export const RAGE_CLICK_WINDOW_MS = 500;

export const getScrollDepth = () => {
  const wh = window.innerHeight;
  const dh = Math.max(
    document.body.scrollHeight, document.body.offsetHeight,
    document.documentElement.scrollHeight, document.documentElement.offsetHeight,
  );
  if (dh <= wh) return 100;
  const st = window.scrollY || document.documentElement.scrollTop;
  return Math.min(100, Math.round((st + wh) / dh * 100));
};

export const calcScrollVelocity = () => {
  if (state.scrollPositions.length < 2) return null;
  const recent = state.scrollPositions.slice(-10);
  let total = 0;
  for (let i = 1; i < recent.length; i++) {
    const td = recent[i].time - recent[i - 1].time;
    if (td > 0) total += Math.abs(recent[i].position - recent[i - 1].position) / td;
  }
  return Math.round(total / (recent.length - 1) * 1000);
};

export const detectReadingPattern = () => {
  const t = Date.now() - state.pageLoadTime;
  const d = state.maxScrollDepth;
  if (t < 10000 && d < 25) return 'bounce';
  if (d > 75 && t > 30000) return 'engaged';
  if (d > 50 && t > 15000) return 'reader';
  if (d > 30 && t < 20000) return 'scanner';
  return 'skimmer';
};

export const recordFirst = () => {
  if (!state.firstInteractionTime) state.firstInteractionTime = Date.now();
};

export const sendEvent = (eventType, data) => {
  const payload = { ...data, page_url: window.location.pathname, event_type: eventType };
  const json = JSON.stringify(payload);
  if (navigator.sendBeacon) {
    navigator.sendBeacon(ENDPOINT, new Blob([json], { type: 'application/json' }));
  } else {
    fetch(ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: json,
      keepalive: true,
    });
  }
};
